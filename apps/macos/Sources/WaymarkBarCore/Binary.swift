import Foundation

/// Finds `waymark` and builds its arguments (see `requirements.md`, "Interfaces & Contracts").
public enum Binary {
    /// Every write tool of `waymark` except `undo`. The app switches them off.
    public static let deniedTools = [
        "add_document", "update_document", "dispute_document", "deprecate_document", "restore_document",
        "delete_document", "move", "create_folder", "delete_folder", "rename_tag", "sync", "init",
        "verify_document", "reject_document",
    ]

    /// The places the app looks in, in order, when the setting is empty.
    public static let searchPaths = ["/opt/homebrew/bin/waymark", "/usr/local/bin/waymark"]

    /// The binary to start. `setting` wins when it is not empty. Returns nil when none exists.
    public static func find(
        setting: String?, fileExists: (String) -> Bool = { FileManager.default.isExecutableFile(atPath: $0) }
    ) -> URL? {
        if let s = setting?.trimmingCharacters(in: .whitespaces), !s.isEmpty {
            let path = (s as NSString).expandingTildeInPath
            return fileExists(path) ? URL(fileURLWithPath: path) : nil
        }
        return searchPaths.first(where: fileExists).map { URL(fileURLWithPath: $0) }
    }

    /// The arguments for the memory repository and the deny list.
    public static func arguments(repository: RepositorySetting, denied: [String] = deniedTools) -> [String] {
        var args = repository.arguments
        var list: [String] = []
        for t in denied where !list.contains(t) { list.append(t) }
        args += ["--tools-denied", list.joined(separator: ",")]
        return args
    }

    /// The environment of the child process. An app that starts from the Finder has a short PATH, and
    /// `waymark` runs `git`.
    public static func environment(base: [String: String] = ProcessInfo.processInfo.environment) -> [String: String] {
        var env = base
        let extra = ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin", "/bin", "/usr/sbin", "/sbin"]
        var parts = (env["PATH"] ?? "").split(separator: ":").map(String.init)
        for p in extra where !parts.contains(p) { parts.append(p) }
        env["PATH"] = parts.joined(separator: ":")
        return env
    }
}

/// The memory repository of the settings: a Git URL or a local folder, as in the MCP entry.
public struct RepositorySetting: Codable, Equatable, Sendable {
    /// A Git URL (`--repo`) or a folder (`--path`).
    public var location: String
    /// A folder for the clone of a URL (`--path` with `--repo`). Optional.
    public var cloneFolder: String
    /// `--memory-repository-id`. Optional.
    public var memoryRepositoryID: String

    public init(location: String = "", cloneFolder: String = "", memoryRepositoryID: String = "") {
        self.location = location
        self.cloneFolder = cloneFolder
        self.memoryRepositoryID = memoryRepositoryID
    }

    public var isEmpty: Bool { location.trimmingCharacters(in: .whitespaces).isEmpty }

    /// Is the location a Git URL (and not a folder)?
    public var isURL: Bool {
        let l = location.trimmingCharacters(in: .whitespaces)
        if l.contains("://") { return true }
        if l.hasPrefix("/") || l.hasPrefix("~") || l.hasPrefix(".") { return false }
        // scp form: user@host:path
        if let at = l.firstIndex(of: "@"), let colon = l.firstIndex(of: ":"), at < colon { return true }
        return false
    }

    public var arguments: [String] {
        let l = location.trimmingCharacters(in: .whitespaces)
        var args: [String] = []
        if isURL {
            args += ["--repo", l]
            let c = cloneFolder.trimmingCharacters(in: .whitespaces)
            if !c.isEmpty { args += ["--path", c] }
        } else {
            args += ["--path", l]
        }
        let id = memoryRepositoryID.trimmingCharacters(in: .whitespaces)
        if !id.isEmpty { args += ["--memory-repository-id", id] }
        return args
    }
}
