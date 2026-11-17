import Foundation

/// An error of the MCP connection itself, not of a tool.
public enum MCPError: Error, Equatable, CustomStringConvertible {
    /// The binary does not exist or cannot run.
    case cannotStart(String)
    /// The process ended. Holds the last lines of its standard error.
    case processExited(String)
    /// A JSON-RPC error answer.
    case rpc(code: Int, message: String)
    /// No answer in time.
    case timeout(String)

    public var description: String {
        switch self {
        case .cannotStart(let s): return "waymark could not start: \(s)"
        case .processExited(let s): return s.isEmpty ? "waymark stopped." : "waymark stopped: \(s)"
        case .rpc(let code, let message): return "MCP error \(code): \(message)"
        case .timeout(let method): return "waymark did not answer \(method) in time."
        }
    }
}

/// A small MCP client over the standard input and output of a child process (see `decisions.md`:
/// "A small MCP client of our own"). One JSON-RPC message per line.
public final class MCPClient: @unchecked Sendable {
    /// Answers a request that the server sends to the client, such as `elicitation/create`.
    /// Returns the `result`, or throws to answer with an error.
    public typealias RequestHandler = @Sendable (_ method: String, _ params: JSON) async throws -> JSON

    private let executable: URL
    private let arguments: [String]
    private let environment: [String: String]
    private let process = Process()
    private let stdin = Pipe()
    private let stdout = Pipe()
    private let stderr = Pipe()
    private let lock = NSLock()
    /// One message at a time on the pipe, so that two writes never interleave.
    private let writeLock = NSLock()
    private var nextID = 1
    private var pending: [Int: CheckedContinuation<JSON, Error>] = [:]
    private var buffer = Data()
    private var errorTail = ""
    private var exited = false
    private var handler: RequestHandler?
    /// The server info of `initialize`.
    public private(set) var serverVersion: String?
    /// Called once when the process ends.
    public var onExit: (@Sendable () -> Void)?

    public init(executable: URL, arguments: [String], environment: [String: String]) {
        self.executable = executable
        self.arguments = arguments
        self.environment = environment
    }

    deinit { terminate() }

    public func setRequestHandler(_ handler: RequestHandler?) {
        lock.withLock { self.handler = handler }
    }

    public var isRunning: Bool { lock.withLock { !exited } && process.isRunning }

    /// Start the process and run the MCP handshake.
    public func start(clientName: String, clientVersion: String, timeout: TimeInterval = 120) async throws {
        process.executableURL = executable
        process.arguments = arguments
        process.environment = environment
        process.standardInput = stdin
        process.standardOutput = stdout
        process.standardError = stderr
        // A write to a pipe whose reader is gone must fail with an error, not stop the app (SIGPIPE).
        _ = fcntl(stdin.fileHandleForWriting.fileDescriptor, F_SETNOSIGPIPE, 1)
        stdout.fileHandleForReading.readabilityHandler = { [weak self] h in
            let data = h.availableData
            if data.isEmpty {
                // End of file: the process is gone. Stop the handler, or it fires again and again.
                h.readabilityHandler = nil
                self?.didExit()
                return
            }
            self?.receive(data)
        }
        stderr.fileHandleForReading.readabilityHandler = { [weak self] h in
            let data = h.availableData
            guard let self else { return }
            if data.isEmpty {
                h.readabilityHandler = nil
                return
            }
            self.lock.withLock {
                self.errorTail = String((self.errorTail + String(decoding: data, as: UTF8.self)).suffix(2000))
            }
        }
        // The end of stdout fails the open requests, after the last answer was read. A short wait
        // covers a process whose stdout stays open (a child that holds it).
        process.terminationHandler = { [weak self] _ in
            DispatchQueue.global().asyncAfter(deadline: .now() + 1) { self?.didExit() }
        }
        do {
            try process.run()
        } catch {
            lock.withLock { exited = true }
            throw MCPError.cannotStart(error.localizedDescription)
        }
        let result = try await request(
            "initialize",
            [
                "protocolVersion": "2025-06-18",
                "capabilities": ["elicitation": ["form": [:]]],
                "clientInfo": ["name": .string(clientName), "version": .string(clientVersion)],
            ], timeout: timeout)
        serverVersion = result["serverInfo"]["version"].string
        try send(["jsonrpc": "2.0", "method": "notifications/initialized"])
    }

    /// Send a request and wait for its answer.
    public func request(_ method: String, _ params: JSON = [:], timeout: TimeInterval = 300) async throws -> JSON {
        let id: Int = lock.withLock {
            defer { nextID += 1 }
            return nextID
        }
        return try await withCheckedThrowingContinuation { (c: CheckedContinuation<JSON, Error>) in
            let failed: Error? = lock.withLock {
                if exited { return MCPError.processExited(lastError) }
                pending[id] = c
                return nil
            }
            if let failed {
                c.resume(throwing: failed)
                return
            }
            do {
                try send(["jsonrpc": "2.0", "id": .number(Double(id)), "method": .string(method), "params": params])
            } catch {
                resume(id, with: .failure(error))
                return
            }
            DispatchQueue.global().asyncAfter(deadline: .now() + timeout) { [weak self] in
                self?.resume(id, with: .failure(MCPError.timeout(method)))
            }
        }
    }

    /// Stop the process. Open requests fail.
    public func terminate() {
        stdout.fileHandleForReading.readabilityHandler = nil
        stderr.fileHandleForReading.readabilityHandler = nil
        if process.isRunning { process.terminate() }
        didExit()
    }

    // MARK: - Internals

    private var lastError: String {
        errorTail.split(separator: "\n").suffix(3).joined(separator: " ").trimmingCharacters(in: .whitespaces)
    }

    private func send(_ message: JSON) throws {
        var data = message.serialized()
        data.append(0x0A)
        do {
            if lock.withLock({ exited }) { throw MCPError.processExited("") }
            try writeLock.withLock { try stdin.fileHandleForWriting.write(contentsOf: data) }
        } catch {
            throw MCPError.processExited(lock.withLock { lastError })
        }
    }

    private func resume(_ id: Int, with result: Result<JSON, Error>) {
        let c = lock.withLock { pending.removeValue(forKey: id) }
        c?.resume(with: result)
    }

    private func didExit() {
        stdout.fileHandleForReading.readabilityHandler = nil
        stderr.fileHandleForReading.readabilityHandler = nil
        let (waiting, message, first): ([CheckedContinuation<JSON, Error>], String, Bool) = lock.withLock {
            let first = !exited
            exited = true
            let w = Array(pending.values)
            pending.removeAll()
            return (w, lastError, first)
        }
        for c in waiting { c.resume(throwing: MCPError.processExited(message)) }
        if first { onExit?() }
    }

    private func receive(_ data: Data) {
        if data.isEmpty { return }
        var lines: [Data] = []
        lock.withLock {
            buffer.append(data)
            while let nl = buffer.firstIndex(of: 0x0A) {
                lines.append(buffer.subdata(in: buffer.startIndex..<nl))
                buffer.removeSubrange(buffer.startIndex...nl)
            }
        }
        for line in lines where !line.isEmpty {
            guard let msg = try? JSON.parse(line) else { continue }
            handle(msg)
        }
    }

    private func handle(_ msg: JSON) {
        let method = msg["method"].string
        let idValue = msg["id"]
        if let method {
            // A request (with an ID) or a notification from the server.
            guard !idValue.isNull else { return }
            let h = lock.withLock { handler }
            Task.detached { [weak self] in
                var reply: JSON = ["jsonrpc": "2.0", "id": idValue]
                if let h {
                    do {
                        reply = ["jsonrpc": "2.0", "id": idValue, "result": try await h(method, msg["params"])]
                    } catch {
                        reply = [
                            "jsonrpc": "2.0", "id": idValue,
                            "error": ["code": -32603, "message": .string(String(describing: error))],
                        ]
                    }
                } else {
                    reply = [
                        "jsonrpc": "2.0", "id": idValue,
                        "error": ["code": -32601, "message": .string("The client does not handle \(method).")],
                    ]
                }
                try? self?.send(reply)
            }
            return
        }
        guard let id = idValue.int else { return }
        if !msg["error"].isNull {
            resume(
                id,
                with: .failure(
                    MCPError.rpc(
                        code: msg["error"]["code"].int ?? 0, message: msg["error"]["message"].string ?? "")))
        } else {
            resume(id, with: .success(msg["result"]))
        }
    }
}
