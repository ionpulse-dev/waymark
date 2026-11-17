import Foundation

/// A tool that answered with an error (`isError`). See `tool-conventions`.
public struct ToolError: Error, Equatable, CustomStringConvertible {
    public var code: String
    public var message: String
    public var next: String?

    public init(code: String, message: String, next: String? = nil) {
        self.code = code
        self.message = message
        self.next = next
    }

    init(_ j: JSON) {
        code = j["code"].string ?? "error"
        message = j["message"].string ?? "waymark reported an error."
        next = j["next"].string
    }

    /// The message and the next step, for a window.
    public var description: String { [message, next].compactMap { $0 }.joined(separator: " ") }
}

/// The effective limits that every result carries (`settings`).
public struct Limits: Equatable, Sendable {
    public var access: String = "read-write"
    public var paused = false
    public var toolsDenied: [String] = []
    public var folders: [String] = []
    public var maxResults = 5
    public var maxDocumentsOpened = 20
    public var approvalRequiredFor: [String] = []
    public var approvalVia = "elicitation"

    public init() {}

    public init?(_ j: JSON) {
        guard j.object != nil else { return nil }
        access = j["access"].string ?? "read-write"
        paused = j["paused"].bool ?? false
        toolsDenied = j["tools_denied"].strings
        folders = j["folders"].strings
        maxResults = j["max_results"].int ?? 5
        maxDocumentsOpened = j["max_documents_opened"].int ?? 20
        approvalRequiredFor = j["approval"]["required_for"].strings
        approvalVia = j["approval"]["via"].string ?? "elicitation"
    }
}

/// Can the person undo a write now? (see "Error states" in `requirements.md`)
public enum UndoState: Equatable, Sendable {
    case allowed
    /// Writes are paused: the button is disabled.
    case paused
    /// Writes are not allowed: the button is hidden. Holds the reason.
    case notAllowed(String)

    public init(_ limits: Limits, deniedByApp: [String] = Binary.deniedTools) {
        if limits.access == "read-only" {
            self = .notAllowed("Writes are not allowed: a setting sets access to read-only.")
        } else if limits.toolsDenied.contains("undo") {
            self = .notAllowed("Undo is switched off in a setting (tools_denied).")
        } else if limits.paused {
            self = .paused
        } else {
            self = .allowed
        }
    }
}

public struct StatusInfo: Equatable, Sendable {
    public var version: String?
    public var healthy: Bool
    public var state: String
    public var name: String?
    public var path: String?
    public var problem: ToolError?
    public var author: String?
    public var limits: Limits

    init(_ j: JSON) {
        version = j["version"].string
        healthy = j["healthy"].bool ?? false
        state = j["memory_repository"]["state"].string ?? "unknown"
        name = j["memory_repository"]["name"].string
        path = j["memory_repository"]["path"].string
        problem = j["problem"].object == nil ? nil : ToolError(j["problem"])
        if let n = j["author"]["name"].string, let e = j["author"]["email"].string {
            author = "\(n) <\(e)>"
        }
        limits = Limits(j["settings"]) ?? Limits()
    }
}

public struct SearchHit: Equatable, Sendable, Identifiable, Hashable {
    public var id: String
    public var title: String
    public var status: String
    public var updated: String
    public var description: String?
    public var snippet: String?

    init(_ j: JSON) {
        id = j["id"].string ?? ""
        title = j["title"].string ?? id
        status = j["status"].string ?? "active"
        updated = j["updated"].string ?? ""
        description = j["description"].string
        snippet = j["snippet"].string
    }

    public init(id: String, title: String, status: String = "active", updated: String, description: String? = nil) {
        self.id = id
        self.title = title
        self.status = status
        self.updated = updated
        self.description = description
    }

    /// The folder of the document: its ID without the last part.
    public var folder: String { DocumentID.folder(of: id) }
}

public struct SearchPage: Equatable, Sendable {
    public var hits: [SearchHit]
    public var total: Int
    public var truncated: Bool

    init(_ j: JSON) {
        hits = j["results"].array.map(SearchHit.init)
        total = j["total_matches"].int ?? hits.count
        truncated = j["truncated"].bool ?? false
    }
}

/// One version of a document, from `show`.
public struct Document: Equatable, Sendable {
    public var id: String
    public var title: String
    public var description: String
    public var body: String
    public var status: String
    public var tags: [String]
    public var updated: String
    public var updatedBy: String
    public var lastEditor: String?
    public var version: String
    public var disputed: Bool
    public var disputeReason: String?
    public var replacedBy: [String]
    public var movedFrom: String?
    public var isCurrent: Bool

    init(_ j: JSON) {
        id = j["id"].string ?? ""
        title = j["title"].string ?? id
        description = j["description"].string ?? ""
        body = j["body"].string ?? ""
        status = j["status"].string ?? "active"
        tags = j["tags"].strings
        updated = j["updated"].string ?? ""
        updatedBy = j["updated_by"].string ?? ""
        lastEditor = j["last_editor"].string
        version = j["version"].string ?? ""
        disputed = j["disputed"].bool ?? false
        disputeReason = j["dispute_reason"].string
        replacedBy = j["replaced_by"].strings
        movedFrom = j["moved_from"].string
        isCurrent = j["is_current"].bool ?? true
    }

    public init(
        id: String, title: String, body: String, version: String, updated: String = "",
        updatedBy: String = "", status: String = "active"
    ) {
        self.id = id
        self.title = title
        description = ""
        self.body = body
        self.status = status
        tags = []
        self.updated = updated
        self.updatedBy = updatedBy
        lastEditor = nil
        self.version = version
        disputed = false
        disputeReason = nil
        replacedBy = []
        movedFrom = nil
        isCurrent = true
    }

    /// The person who changed it last: the commit author, else `updated_by`.
    public var changedBy: String { Person.name(lastEditor ?? updatedBy) }

    /// The properties that the diff compares.
    public var properties: [(String, String)] {
        [
            ("Title", title), ("Description", description), ("Status", status),
            ("Tags", tags.joined(separator: ", ")),
            ("Disputed", disputed ? (disputeReason ?? "yes") : "no"),
        ]
    }
}

public struct HistoryEntry: Equatable, Sendable {
    public var version: String
    public var verb: String
    public var author: String
    public var time: String
    public var subject: String

    init(_ j: JSON) {
        version = j["version"].string ?? ""
        verb = j["verb"].string ?? ""
        author = j["author"].string ?? ""
        time = j["time"].string ?? ""
        subject = j["subject"].string ?? ""
    }
}

public struct HistoryPage: Equatable, Sendable {
    public var entries: [HistoryEntry]
    public var nextBefore: String?

    init(_ j: JSON) {
        entries = j["entries"].array.map(HistoryEntry.init)
        nextBefore = j["next_before"].string
    }
}

/// One write of the person, from `activity`.
public struct ActivityEntry: Equatable, Sendable, Identifiable, Hashable {
    public var commit: String
    public var verb: String
    public var title: String
    public var folder: String
    public var ids: [String]
    public var time: String
    public var undone: Bool
    public var laterChanges: Bool
    public var files: Int

    public var id: String { commit }

    init(_ j: JSON) {
        commit = j["commit"].string ?? ""
        verb = j["verb"].string ?? ""
        title = j["title"].string ?? ""
        folder = j["folder"].string ?? ""
        ids = j["ids"].strings
        time = j["time"].string ?? ""
        undone = j["undone"].bool ?? false
        laterChanges = j["later_changes"].bool ?? false
        files = j["files"].int ?? ids.count
    }

    public init(
        commit: String, verb: String, title: String, folder: String, ids: [String], time: String = "",
        undone: Bool = false
    ) {
        self.commit = commit
        self.verb = verb
        self.title = title
        self.folder = folder
        self.ids = ids
        self.time = time
        self.undone = undone
        laterChanges = false
        files = ids.count
    }

    public var date: Date? { Time.parse(time) }
}

public struct ActivityPage: Equatable, Sendable {
    public var entries: [ActivityEntry]
    public var nextBefore: String?

    init(_ j: JSON) {
        entries = j["entries"].array.map(ActivityEntry.init)
        nextBefore = j["next_before"].string
    }
}

/// One change of another person, from `updates`.
public struct UpdateChange: Equatable, Sendable {
    public var id: String
    public var kind: String
    public var title: String
    public var by: String
    public var from: String?

    init(_ j: JSON) {
        id = j["id"].string ?? ""
        kind = j["change"].string ?? j["kind"].string ?? "updated"
        title = j["title"].string ?? id
        let author = j["by"].string ?? j["author"].string ?? j["authors"][0].string ?? ""
        by = Person.name(author)
        from = j["from"].string
    }

    public init(id: String, kind: String = "updated", title: String = "", by: String = "") {
        self.id = id
        self.kind = kind
        self.title = title
        self.by = by
        from = nil
    }
}

public struct UpdatesResult: Equatable, Sendable {
    public var changes: [UpdateChange]
    public var until: String?

    init(_ j: JSON) {
        changes = j["changes"].array.map(UpdateChange.init)
        until = j["until"].string
    }
}

/// The result of `undo`, with or without `preview`.
public struct UndoResult: Equatable, Sendable {
    public struct Change: Equatable, Sendable, Hashable {
        public var action: String
        public var id: String
    }

    public var changes: [Change]
    public var message: String
    public var commit: String?
    public var pushed: Bool?
    public var preview: Bool

    init(_ j: JSON) {
        changes = j["changes"].array.map { Change(action: $0["action"].string ?? "", id: $0["id"].string ?? "") }
        message = j["message"].string ?? ""
        commit = j["commit"].string
        pushed = j["pushed"].bool
        preview = j["preview"].bool ?? false
    }
}

/// Helpers for names in the form `Jane Doe <jane@example.org>`.
public enum Person {
    public static func name(_ s: String) -> String {
        if let i = s.firstIndex(of: "<") {
            return s[..<i].trimmingCharacters(in: .whitespaces)
        }
        return s
    }
}

public enum DocumentID {
    public static func folder(of id: String) -> String {
        guard let i = id.lastIndex(of: "/") else { return "" }
        return String(id[..<i])
    }

    /// Is `id` in `folder` or one of its subfolders? The root folder `""` holds all.
    public static func isInside(_ id: String, folder: String) -> Bool {
        let f = folder.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
        return f.isEmpty || id.hasPrefix(f + "/")
    }

    /// Is a folder `inner` the folder `outer` or inside it?
    public static func folderIsInside(_ inner: String, _ outer: String) -> Bool {
        let o = outer.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
        return o.isEmpty || inner == o || inner.hasPrefix(o + "/")
    }
}

public enum Time {
    /// Parse an RFC 3339 time (with or without fractions) or a date.
    public static func parse(_ s: String) -> Date? {
        let f = ISO8601DateFormatter()
        f.formatOptions = [.withInternetDateTime]
        if let d = f.date(from: s) { return d }
        f.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        if let d = f.date(from: s) { return d }
        f.formatOptions = [.withFullDate]
        return f.date(from: s)
    }

    /// An RFC 3339 time in UTC, for `since`.
    public static func rfc3339(_ d: Date) -> String {
        let f = ISO8601DateFormatter()
        f.formatOptions = [.withInternetDateTime]
        f.timeZone = TimeZone(identifier: "UTC")
        return f.string(from: d)
    }
}
