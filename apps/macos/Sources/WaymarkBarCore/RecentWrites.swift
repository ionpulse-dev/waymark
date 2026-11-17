import Foundation

/// The change of one write to one document (see `plan.md`, step 7).
public struct WriteChange: Equatable, Sendable {
    public enum Shape: Equatable, Sendable {
        /// A new document: only the version of the write.
        case added(Document)
        /// A change: the version before and the version of the write.
        case changed(before: Document, after: Document, lines: [Diff.Line], properties: [Diff.Property])
        /// A move: the new path. `history` and `show` follow the document.
        case moved(to: String)
        /// A delete: no text, `show` does not read a deleted document.
        case deleted
        /// The versions could not be read. Holds the reason.
        case unavailable(String)
    }

    public var id: String
    public var shape: Shape
}

public enum RecentWrites {
    /// Same commit? `activity` and `history` both give short hashes, maybe of other lengths.
    public static func same(_ a: String, _ b: String) -> Bool {
        !a.isEmpty && !b.isEmpty && (a.hasPrefix(b) || b.hasPrefix(a))
    }

    /// Read `activity` until the newest known write (at most `maxPages` pages). Returns the merged list,
    /// newest first, and the commits that are new.
    public static func refresh(
        _ caller: ToolCaller, known: [ActivityEntry], maxPages: Int = 5
    ) async throws -> (entries: [ActivityEntry], new: [String], notices: [String]) {
        var fresh: [ActivityEntry] = []
        var notices: [String] = []
        var before: String?
        let newestKnown = known.first?.commit
        var reachedKnown = newestKnown == nil
        for page in 0..<maxPages {
            let r = try await caller.activity(before: before, limit: 50)
            notices += r.notices
            for e in r.value.entries {
                if let k = newestKnown, same(e.commit, k) { reachedKnown = true }
                fresh.append(e)
            }
            // The first start reads one page only.
            if newestKnown == nil || reachedKnown || page == maxPages - 1 { break }
            guard let next = r.value.nextBefore else { break }
            before = next
        }
        _ = reachedKnown
        let knownSet = Set(known.map(\.commit))
        let freshSet = Set(fresh.map(\.commit))
        let new = fresh.filter { !knownSet.contains($0.commit) }.map(\.commit)
        // Fresh entries carry the current flags (undone, later changes). Keep older known entries.
        let merged = fresh + known.filter { !freshSet.contains($0.commit) }
        return (merged, new, notices)
    }

    /// The change of `entry` to the document `id`.
    public static func change(of entry: ActivityEntry, id: String, caller: ToolCaller) async -> WriteChange {
        switch entry.verb {
        case "delete", "delete_folder":
            return WriteChange(id: id, shape: .deleted)
        case "move":
            return WriteChange(id: id, shape: .moved(to: id))
        default:
            break
        }
        do {
            let after = try await caller.show(id, version: entry.commit).value
            if entry.verb == "add" {
                return WriteChange(id: id, shape: .added(after))
            }
            guard let beforeVersion = try await versionBefore(entry.commit, id: id, caller: caller) else {
                return WriteChange(id: id, shape: .added(after))
            }
            let before = try await caller.show(id, version: beforeVersion).value
            return WriteChange(
                id: id,
                shape: .changed(
                    before: before, after: after, lines: Diff.lines(before.body, after.body),
                    properties: Diff.properties(before, after)))
        } catch let e as ToolError {
            return WriteChange(id: id, shape: .unavailable(e.description))
        } catch {
            return WriteChange(id: id, shape: .unavailable(String(describing: error)))
        }
    }

    /// The version of `id` just before `commit`, from `history`. Nil when the write made the document.
    public static func versionBefore(_ commit: String, id: String, caller: ToolCaller) async throws -> String? {
        var before: String?
        var found = false
        for _ in 0..<20 {
            let page = try await caller.history(id, before: before, limit: 50).value
            for e in page.entries {
                if found { return e.version }
                if same(e.version, commit) { found = true }
            }
            guard let next = page.nextBefore else { return nil }
            before = next
        }
        return nil
    }
}
