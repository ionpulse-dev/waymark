import Foundation

/// The state of the menu bar icon (see "Menu" in `requirements.md`).
public enum IconState: Equatable, Sendable {
    case normal
    /// The last poll had the notice that the clone may be behind.
    case behind
    /// `waymark` is missing, did not start, or reported a problem.
    case error(String)
}

/// What one poll found for a watched document.
public enum DocumentPoll: Equatable, Sendable {
    /// The first read, or a read with the same version.
    case loaded(Document)
    /// A new version since the last poll.
    case changed(Document)
    /// `show` followed a move: the document has a new ID.
    case moved(Document)
    /// The document is gone.
    case deleted(ToolError)
    /// The `folders` limit hides it, or another tool error.
    case unavailable(ToolError)
}

/// The list of a watched folder.
public struct FolderListing: Equatable, Sendable {
    /// Newest first.
    public var hits: [SearchHit]
    public var total: Int

    public init(hits: [SearchHit], total: Int) {
        self.hits = hits
        self.total = total
    }
}

/// What one poll found for a watched folder.
public struct FolderPoll: Equatable, Sendable {
    public var listing: FolderListing?
    /// IDs that are new since the last full read.
    public var newIDs: Set<String> = []
    /// IDs that changed since the last full read.
    public var changedIDs: Set<String> = []
    /// The names of the people who changed it, from `updates`.
    public var changedBy: [String] = []
    /// The `folders` limit hides it, or another tool error.
    public var error: ToolError?
    /// Did this poll change something?
    public var hasChanges: Bool { !newIDs.isEmpty || !changedIDs.isEmpty }
}

public struct PollResult: Equatable, Sendable {
    public var documents: [String: DocumentPoll] = [:]
    public var folders: [String: FolderPoll] = [:]
    /// All known writes of the person, newest first.
    public var activity: [ActivityEntry] = []
    /// The writes that this poll found for the first time.
    public var newWrites: [String] = []
    public var state: IconState = .normal
    public var notices: [String] = []
    public var limits: Limits?
    /// The error of the poll, if it stopped early. The last content stays.
    public var error: String?
}

/// One poll after the other (see `plan.md`, steps 4 to 6). Holds what the last poll saw.
public actor Poller {
    private let caller: ToolCaller
    private var versions: [String: String] = [:]
    private var listings: [String: FolderListing] = [:]
    private var activity: [ActivityEntry] = []
    private var activityLoaded = false
    private var lastPollStart: Date?
    /// Changes from `updates` that were already reported: key and the time they were seen.
    private var reported: [String: Date] = [:]
    /// `updates` looks back this far before the last poll, for commits that were pushed late.
    public static let overlap: TimeInterval = 600

    public init(caller: ToolCaller, startedAt: Date = Date()) {
        self.caller = caller
        lastPollStart = startedAt
    }

    public var knownActivity: [ActivityEntry] { activity }

    /// Run one poll. `fullFolders` are read in full in any case (window opened, Refresh now).
    public func poll(_ watched: [WatchedItem], fullFolders: Set<String> = [], now: Date = Date()) async -> PollResult {
        var result = PollResult()
        let since = (lastPollStart ?? now).addingTimeInterval(-Self.overlap)
        reported = reported.filter { now.timeIntervalSince($0.value) < Self.overlap * 2 }
        do {
            // 1. The writes of the person.
            let r = try await RecentWrites.refresh(caller, known: activity)
            result.notices += r.notices
            let newEntries = activityLoaded ? r.entries.filter { r.new.contains($0.commit) } : []
            result.newWrites = activityLoaded ? r.new : []
            activity = r.entries
            activityLoaded = true

            // 2. Watched folders.
            for item in watched where item.kind == .folder {
                result.folders[item.path] = try await pollFolder(
                    item.path, since: since, ownWrites: newEntries, full: fullFolders.contains(item.path),
                    notices: &result.notices, now: now)
            }
            // 3. Watched documents.
            for item in watched where item.kind == .document {
                result.documents[item.path] = try await pollDocument(item.path, notices: &result.notices)
            }
            lastPollStart = now
        } catch let e as ToolError {
            result.error = e.description
            result.state = .error(e.description)
        } catch {
            let text = String(describing: error)
            result.error = text
            result.state = .error(text)
        }
        result.activity = activity
        if case .normal = result.state, result.notices.contains(where: Self.isBehind) {
            result.state = .behind
        }
        return result
    }

    /// Read a folder in full now (its window opened).
    public func readFolder(_ path: String) async throws -> FolderListing {
        let l = try await fullRead(path)
        listings[path] = l
        return l
    }

    /// Forget what was seen of an item (it was removed from the watched list).
    public func forget(_ item: WatchedItem) {
        switch item.kind {
        case .document: versions[item.path] = nil
        case .folder: listings[item.path] = nil
        }
    }

    /// Set the undone flag after an undo, also for an entry that is older than the pages read.
    public func markUndone(_ commits: [String]) {
        activity = activity.map { e in
            var e = e
            if commits.contains(where: { RecentWrites.same($0, e.commit) }) { e.undone = true }
            return e
        }
    }

    /// Add older writes ("Load more").
    public func loadMore() async throws -> [ActivityEntry] {
        guard let last = activity.last?.commit else { return activity }
        let page = try await caller.activity(before: last, limit: 50).value
        let known = Set(activity.map(\.commit))
        activity += page.entries.filter { !known.contains($0.commit) }
        return activity
    }

    static func isBehind(_ notice: String) -> Bool {
        notice.localizedCaseInsensitiveContains("may be behind")
            || notice.localizedCaseInsensitiveContains("cannot reach the remote")
    }

    // MARK: - Internals

    private func pollDocument(_ id: String, notices: inout [String]) async throws -> DocumentPoll {
        do {
            let r = try await caller.show(id)
            notices += r.notices
            let doc = r.value
            let old = versions[id]
            versions[id] = doc.version
            if doc.id != id {
                versions[doc.id] = doc.version
                return .moved(doc)
            }
            if let old, old != doc.version { return .changed(doc) }
            return .loaded(doc)
        } catch let e as ToolError {
            switch e.code {
            case "not_found": return .deleted(e)
            case "access_denied", "invalid_arguments": return .unavailable(e)
            default: throw e
            }
        }
    }

    private func pollFolder(
        _ path: String, since: Date, ownWrites: [ActivityEntry], full: Bool, notices: inout [String], now: Date
    ) async throws -> FolderPoll {
        var poll = FolderPoll()
        do {
            var hinted: Set<String> = []
            var names: [String] = []
            if listings[path] != nil {
                let u = try await caller.updates(since: since, folders: path.isEmpty ? [] : [path])
                notices += u.notices
                for c in u.value.changes where DocumentID.isInside(c.id, folder: path) {
                    let key = "\(c.id)|\(c.kind)|\(c.by)|\(c.from ?? "")"
                    if reported[key] != nil { continue }
                    reported[key] = now
                    hinted.insert(c.id)
                    if !c.by.isEmpty && !names.contains(c.by) { names.append(c.by) }
                }
                for e in ownWrites where DocumentID.folderIsInside(e.folder, path)
                    || e.ids.contains(where: { DocumentID.isInside($0, folder: path) })
                {
                    hinted.formUnion(e.ids)
                }
            }
            guard listings[path] == nil || full || !hinted.isEmpty else {
                poll.listing = listings[path]
                return poll
            }
            let old = listings[path]
            let fresh = try await fullRead(path)
            listings[path] = fresh
            poll.listing = fresh
            poll.changedBy = names
            if let old {
                let before = Dictionary(old.hits.map { ($0.id, $0) }, uniquingKeysWith: { a, _ in a })
                for h in fresh.hits {
                    if let b = before[h.id] {
                        if b.updated != h.updated || b.title != h.title || b.status != h.status
                            || hinted.contains(h.id)
                        {
                            poll.changedIDs.insert(h.id)
                        }
                    } else {
                        poll.newIDs.insert(h.id)
                    }
                }
            }
            return poll
        } catch let e as ToolError where e.code == "access_denied" || e.code == "invalid_arguments" {
            poll.error = e
            return poll
        }
    }

    private func fullRead(_ path: String) async throws -> FolderListing {
        var hits: [SearchHit] = []
        var total = 0
        var offset = 0
        for _ in 0..<400 {
            let r = try await caller.search(
                folders: path.isEmpty ? [] : [path], level: 1, status: "any", offset: offset)
            total = r.value.total
            if r.value.hits.isEmpty { break }
            hits += r.value.hits
            offset += r.value.hits.count
            if offset >= total { break }
        }
        // Newest first. `updated` is a date. Ties keep the title order.
        hits.sort { ($0.updated, $1.title) > ($1.updated, $0.title) }
        return FolderListing(hits: hits, total: total)
    }
}
