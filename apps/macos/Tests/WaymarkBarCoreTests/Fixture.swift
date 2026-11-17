import Foundation
import XCTest
@testable import WaymarkBarCore

/// A temporary memory repository: a bare remote and two clones, A (the person of the app) and B
/// (another person). `HOME` is a temporary folder, so no personal config is read.
final class Fixture {
    enum Who { case a, b }

    let root: URL
    let remote: URL
    let a: URL
    let b: URL
    let home: URL
    let binary: URL

    static var waymark: URL? {
        if let p = ProcessInfo.processInfo.environment["WAYMARK_BIN"], !p.isEmpty {
            return URL(fileURLWithPath: p)
        }
        // Tests/WaymarkBarCoreTests/Fixture.swift -> the repository root.
        let root = URL(fileURLWithPath: #filePath).deletingLastPathComponent().deletingLastPathComponent()
            .deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        let u = root.appendingPathComponent("target/debug/waymark")
        return FileManager.default.isExecutableFile(atPath: u.path) ? u : nil
    }

    init() throws {
        guard let bin = Fixture.waymark else {
            throw XCTSkip("No waymark binary. Run `cargo build` or set WAYMARK_BIN.")
        }
        binary = bin
        root = FileManager.default.temporaryDirectory.appendingPathComponent("waymark-bar-\(UUID().uuidString)")
            .resolvingSymlinksInPath()
        remote = root.appendingPathComponent("remote.git")
        a = root.appendingPathComponent("a")
        b = root.appendingPathComponent("b")
        home = root.appendingPathComponent("home")
        try FileManager.default.createDirectory(at: home, withIntermediateDirectories: true)
        try git(root, "init", "-q", "--bare", "-b", "main", remote.path)
        try git(root, "clone", "-q", remote.path, a.path)
        try git(a, "-c", "user.name=Setup", "-c", "user.email=setup@example.org", "commit", "-q", "--allow-empty", "-m", "start")
        try git(a, "push", "-q", "origin", "HEAD:main")
        try git(root, "clone", "-q", remote.path, b.path)
        for (dir, name, mail) in [(a, "Anna Example", "anna@example.org"), (b, "Bob Roe", "bob@example.org")] {
            try git(dir, "config", "user.name", name)
            try git(dir, "config", "user.email", mail)
        }
    }

    deinit { try? FileManager.default.removeItem(at: root) }

    var environment: [String: String] {
        var env = Binary.environment()
        env["HOME"] = home.path
        env["XDG_CONFIG_HOME"] = home.appendingPathComponent(".config").path
        env["WAYMARK_SYSTEM_CONFIG"] = root.appendingPathComponent("no-system-config.toml").path
        env["GIT_CONFIG_NOSYSTEM"] = "1"
        return env
    }

    /// The session of the app, as the app starts it, for person A.
    func appSession() -> Session {
        Session(
            config: Session.Config(
                binarySetting: binary.path, repository: RepositorySetting(location: a.path),
                environment: environment))
    }

    /// An agent of a person: a `waymark` with every tool, for setup and for writes.
    func agent(_ who: Who) async throws -> Agent {
        let c = MCPClient(executable: binary, arguments: ["--path", (who == .a ? a : b).path], environment: environment)
        try await c.start(clientName: "test-agent", clientVersion: "0")
        return Agent(client: c)
    }

    /// `init` the memory repository from A's agent.
    func initialize() async throws {
        let ag = try await agent(.a)
        _ = try await ag.call("init", ["name": "Test memory"])
        ag.client.terminate()
        // B cloned before the init. A waymark that starts in a clone with commits but no memory
        // repository does not pick up a later init of the remote (see git-sync/todo.md).
        try git(b, "pull", "-q")
    }

    /// Write the personal config file of A (and B: they share HOME).
    func personalConfig(_ text: String) throws {
        let dir = home.appendingPathComponent(".config/waymark")
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        try text.write(to: dir.appendingPathComponent("config.toml"), atomically: true, encoding: .utf8)
    }

    @discardableResult
    func git(_ dir: URL, _ args: String...) throws -> String {
        let p = Process()
        p.executableURL = URL(fileURLWithPath: "/usr/bin/git")
        p.arguments = ["-C", dir.path] + args
        p.environment = environment
        let out = Pipe()
        p.standardOutput = out
        p.standardError = Pipe()
        try p.run()
        let data = out.fileHandleForReading.readDataToEndOfFile()
        p.waitUntilExit()
        guard p.terminationStatus == 0 else {
            throw NSError(domain: "git", code: Int(p.terminationStatus), userInfo: [NSLocalizedDescriptionKey: "git \(args)"])
        }
        return String(decoding: data, as: UTF8.self)
    }

    func remoteHead() throws -> String {
        try git(remote, "rev-parse", "main").trimmingCharacters(in: .whitespacesAndNewlines)
    }
}

/// A test agent with every tool.
struct Agent: ToolCaller {
    let client: MCPClient

    func call(_ tool: String, _ arguments: JSON) async throws -> JSON {
        let r = try await client.request("tools/call", ["name": .string(tool), "arguments": arguments])
        let c = r["structuredContent"]
        if r["isError"].bool == true { throw ToolError(c) }
        return c
    }

    @discardableResult
    func add(_ folder: String, _ title: String, body: String = "Text.") async throws -> JSON {
        try await call(
            "add_document",
            ["folder": .string(folder), "title": .string(title), "description": "A test document", "body": .string(body)])
    }

    @discardableResult
    func update(_ id: String, body: String) async throws -> JSON {
        let base = try await show(id).value.version
        return try await call("update_document", ["id": .string(id), "base": .string(base), "body": .string(body)])
    }
}
