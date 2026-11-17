import Foundation

/// An answer of a tool, with the notices and the limits that every result carries.
public struct Reply<T: Sendable>: Sendable {
    public var value: T
    public var notices: [String]
    public var limits: Limits?

    public init(value: T, notices: [String] = [], limits: Limits? = nil) {
        self.value = value
        self.notices = notices
        self.limits = limits
    }
}

/// Calls one MCP tool and returns its `structuredContent`. Throws `ToolError` for a tool error.
/// `Session` implements it with a real `waymark`, the tests with a fake.
public protocol ToolCaller: Sendable {
    func call(_ tool: String, _ arguments: JSON) async throws -> JSON
}

extension ToolCaller {
    private func reply<T>(_ tool: String, _ args: JSON, _ make: (JSON) -> T) async throws -> Reply<T> {
        let j = try await call(tool, args)
        return Reply(value: make(j), notices: j["notices"].strings, limits: Limits(j["settings"]))
    }

    public func status() async throws -> Reply<StatusInfo> {
        try await reply("status", [:], StatusInfo.init)
    }

    public func search(
        query: String? = nil, folders: [String] = [], level: Int, status: String? = nil, offset: Int = 0,
        limit: Int? = nil
    ) async throws -> Reply<SearchPage> {
        var a: [String: JSON] = ["level": .number(Double(level))]
        if let query, !query.isEmpty { a["query"] = .string(query) }
        if !folders.isEmpty { a["folders"] = JSON(folders) }
        if let status { a["status"] = .string(status) }
        if offset > 0 { a["offset"] = .number(Double(offset)) }
        if let limit { a["limit"] = .number(Double(limit)) }
        return try await reply("search", .object(a), SearchPage.init)
    }

    public func show(_ id: String, version: String? = nil) async throws -> Reply<Document> {
        var a: [String: JSON] = ["id": .string(id)]
        if let version { a["version"] = .string(version) }
        return try await reply("show", .object(a), Document.init)
    }

    public func history(_ id: String, before: String? = nil, limit: Int = 50) async throws -> Reply<HistoryPage> {
        var a: [String: JSON] = ["id": .string(id), "limit": .number(Double(limit))]
        if let before { a["before"] = .string(before) }
        return try await reply("history", .object(a), HistoryPage.init)
    }

    public func activity(before: String? = nil, limit: Int = 50) async throws -> Reply<ActivityPage> {
        var a: [String: JSON] = ["limit": .number(Double(limit))]
        if let before { a["before"] = .string(before) }
        return try await reply("activity", .object(a), ActivityPage.init)
    }

    /// Always with `since`: without it, `updates` would move the stored time of the agent tools.
    public func updates(since: Date, folders: [String], offset: Int = 0) async throws -> Reply<UpdatesResult> {
        var a: [String: JSON] = ["since": .string(Time.rfc3339(since))]
        if offset > 0 { a["offset"] = .number(Double(offset)) }
        if !folders.isEmpty { a["folders"] = JSON(folders) }
        return try await reply("updates", .object(a), UpdatesResult.init)
    }

    public func undo(commit: String, preview: Bool) async throws -> Reply<UndoResult> {
        try await reply("undo", ["commit": .string(commit), "preview": .bool(preview)], UndoResult.init)
    }
}

public enum SessionError: Error, Equatable, CustomStringConvertible {
    /// No `waymark` binary was found.
    case notInstalled
    /// No memory repository in the settings.
    case notConfigured
    /// The process failed. The next start waits until the given time (back-off).
    case waiting(until: Date, reason: String)

    public var description: String {
        switch self {
        case .notInstalled: return "waymark is not installed. Install it with Homebrew, then choose Retry."
        case .notConfigured: return "No memory repository is set. Choose one in Settings."
        case .waiting(_, let reason): return reason
        }
    }
}

/// One `waymark` process for the app (see `plan.md`, step 3).
public actor Session: ToolCaller {
    public struct Config: Sendable, Equatable {
        public var binarySetting: String?
        public var repository: RepositorySetting
        public var environment: [String: String]
        public var clientVersion: String

        public init(
            binarySetting: String? = nil, repository: RepositorySetting,
            environment: [String: String] = Binary.environment(), clientVersion: String = "1.0"
        ) {
            self.binarySetting = binarySetting
            self.repository = repository
            self.environment = environment
            self.clientVersion = clientVersion
        }
    }

    public let config: Config
    private let findBinary: @Sendable (String?) -> URL?
    private var client: MCPClient?
    /// Clients replaced by a restart. Each stops when its last call ends.
    private var retired: [MCPClient] = []
    /// Open calls per client.
    private var inFlight: [ObjectIdentifier: Int] = [:]
    /// The start that runs now. Callers that come meanwhile wait for it.
    private var starting: Task<Void, Error>?
    private var denied: [String] = Binary.deniedTools
    /// The documents that `show` opened in this process. `waymark` counts each ID once.
    private var opened: Set<String> = []
    private var failures = 0
    private var notBefore: Date?
    private var elicitation: MCPClient.RequestHandler?
    /// The limits from the last result.
    public private(set) var limits = Limits()
    /// The tools that the running process lists, with their `readOnlyHint`.
    public private(set) var tools: [String: Bool] = [:]
    /// How many processes this session started (for tests).
    public private(set) var starts = 0
    public private(set) var serverVersion: String?

    public init(config: Config, findBinary: @escaping @Sendable (String?) -> URL? = { Binary.find(setting: $0) }) {
        self.config = config
        self.findBinary = findBinary
    }

    /// The handler for `elicitation/create` (the approval of `write-approval`).
    public func setElicitationHandler(_ h: MCPClient.RequestHandler?) {
        elicitation = h
        client?.setRequestHandler(h)
    }

    /// The deny list in use (it grows when a newer `waymark` lists a new write tool).
    public var deniedTools: [String] { denied }

    /// Stop every process of this session.
    public func stop() {
        starting?.cancel()
        starting = nil
        client?.terminate()
        client = nil
        for c in retired { c.terminate() }
        retired = []
    }

    /// Start a new process (a new session for the read budget). Calls that run on the old process
    /// finish there first.
    public func restart() async throws {
        if let old = client {
            retired.append(old)
            client = nil
            stopIdleRetired()
        }
        notBefore = nil
        try await ensureStarted()
    }

    public func call(_ tool: String, _ arguments: JSON) async throws -> JSON {
        try await ensureStarted()
        let id = tool == "show" ? arguments["id"].string : nil
        if let id, !opened.contains(id), opened.count >= max(1, limits.maxDocumentsOpened) {
            try await restart()
        }
        do {
            let j = try await rawCall(tool, arguments)
            if let id { opened.insert(id) }
            return j
        } catch let e as ToolError where e.code == "access_denied" && tool == "show" && !opened.isEmpty {
            // The read budget of the session is used up: one new session, one more try.
            try await restart()
            let j = try await rawCall(tool, arguments)
            if let id { opened.insert(id) }
            return j
        }
    }

    // MARK: - Internals

    private func rawCall(_ tool: String, _ arguments: JSON) async throws -> JSON {
        guard let c = client else { throw MCPError.processExited("") }
        let key = ObjectIdentifier(c)
        inFlight[key, default: 0] += 1
        defer {
            inFlight[key, default: 1] -= 1
            if inFlight[key] == 0 { inFlight[key] = nil }
            stopIdleRetired()
        }
        let r: JSON
        do {
            r = try await c.request("tools/call", ["name": .string(tool), "arguments": arguments])
        } catch let e as MCPError {
            if case .processExited = e { processFailed(c) }
            throw e
        }
        let content = r["structuredContent"].object != nil ? r["structuredContent"] : Self.textContent(r)
        if let l = Limits(content["settings"]) { limits = l }
        if r["isError"].bool == true { throw ToolError(content) }
        return content
    }

    private func stopIdleRetired() {
        retired.removeAll { c in
            if inFlight[ObjectIdentifier(c)] == nil {
                c.terminate()
                return true
            }
            return false
        }
    }

    private static func textContent(_ r: JSON) -> JSON {
        for c in r["content"].array {
            if let t = c["text"].string, let j = try? JSON.parse(t) { return j }
        }
        return .null
    }

    /// The process `c` failed. Only the current client counts: an old one may end after a restart.
    private func processFailed(_ c: MCPClient?, starting: Bool = false) {
        c?.terminate()
        guard starting || c === client else { return }
        client = nil
        failures += 1
        let delay = min(60.0, pow(2.0, Double(failures - 1)))
        notBefore = Date().addingTimeInterval(delay)
    }

    private func ensureStarted() async throws {
        if let client, client.isRunning { return }
        if let starting { return try await starting.value }
        let t = Task { try await self.startProcess() }
        starting = t
        defer { starting = nil }
        try await t.value
    }

    private func startProcess() async throws {
        client = nil
        if config.repository.isEmpty { throw SessionError.notConfigured }
        guard let binary = findBinary(config.binarySetting) else { throw SessionError.notInstalled }
        if let nb = notBefore, nb > Date() {
            throw SessionError.waiting(until: nb, reason: "waymark stopped. The app tries again soon.")
        }
        for _ in 0..<3 {
            let c = MCPClient(
                executable: binary, arguments: Binary.arguments(repository: config.repository, denied: denied),
                environment: config.environment)
            c.setRequestHandler(elicitation)
            starts += 1
            var listed: [String: Bool] = [:]
            do {
                try await c.start(clientName: "waymark-bar", clientVersion: config.clientVersion)
                // Check the switched-off tools. A newer waymark may have a new write tool.
                var cursor: String?
                repeat {
                    let page = try await c.request("tools/list", cursor.map { ["cursor": .string($0)] } ?? [:])
                    for t in page["tools"].array {
                        if let name = t["name"].string {
                            listed[name] = t["annotations"]["readOnlyHint"].bool ?? false
                        }
                    }
                    cursor = page["nextCursor"].string
                } while cursor != nil
            } catch {
                processFailed(c, starting: true)
                throw error
            }
            let extra = listed.filter { !$0.value && $0.key != "undo" && !denied.contains($0.key) }.map(\.key).sorted()
            if !extra.isEmpty {
                denied += extra
                c.terminate()
                continue
            }
            client = c
            tools = listed
            serverVersion = c.serverVersion
            opened = []
            failures = 0
            notBefore = nil
            if let s = try? await rawCall("status", [:]), let l = Limits(s["settings"]) { limits = l }
            return
        }
        throw MCPError.cannotStart("The tool list of waymark kept changing.")
    }
}
