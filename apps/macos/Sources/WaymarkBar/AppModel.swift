import AppKit
import Foundation
import Observation
import ServiceManagement
import SwiftUI
import UserNotifications
import WaymarkBarCore

/// What the app knows about one watched or opened document.
struct DocumentState: Equatable {
    var document: Document?
    /// The last change seen by a poll, and who made it.
    var changedAt: Date?
    var changedBy: String?
    /// The document is gone, or the `folders` limit hides it.
    var problem: String?
    /// `show` followed a move to this ID.
    var movedTo: String?
}

/// What the app knows about one watched folder.
struct FolderState: Equatable {
    var listing: FolderListing?
    /// Marks since the window was last shown.
    var newIDs: Set<String> = []
    var changedIDs: Set<String> = []
    var changedBy: [String] = []
    var problem: String?
}

enum Setup: Equatable {
    case ready
    case needsRepository
    case notInstalled
    case problem(String)
}

/// The state of the app (see `plan.md`). All UI reads it on the main actor.
@MainActor
@Observable
final class AppModel {
    static let shared = AppModel()

    private let settingsStore = SettingsStore()
    private let watchedStore = WatchedStore()
    let reviewed = ReviewedStore()

    var settings: AppSettings { didSet { settingsChanged(from: oldValue) } }
    private(set) var watched: [WatchedItem]
    private(set) var icon: IconState = .normal
    private(set) var setup: Setup = .ready
    private(set) var lastPoll: Date?
    private(set) var polling = false
    private(set) var documents: [String: DocumentState] = [:]
    private(set) var folders: [String: FolderState] = [:]
    /// Watched items that changed since their window was last shown.
    private(set) var dirty: Set<String> = []
    private(set) var activity: [ActivityEntry] = []
    /// Bumped when the reviewed set changes, so views update.
    private(set) var reviewedVersion = 0
    private(set) var undoState: UndoState = .allowed
    private(set) var limits = Limits()
    private(set) var serverVersion: String?

    /// The windows that are open now: "document:<id>", "folder:<path>", "activity".
    var visible: Set<String> = []
    /// Set by a view at start, so that a notification can open a window.
    var openWindowAction: OpenWindowAction?

    private(set) var session: Session?
    private var poller: Poller?
    private var loop: Task<Void, Never>?
    private var pendingFull: Set<String> = []

    private init() {
        settings = settingsStore.settings
        watched = watchedStore.items
    }

    // MARK: - Start and the poll loop

    func start() {
        guard loop == nil else { return }
        rebuildSession()
        loop = Task { [weak self] in
            while !Task.isCancelled {
                guard let self else { return }
                await self.pollNow()
                let seconds = self.settings.clampedInterval
                try? await Task.sleep(nanoseconds: UInt64(seconds * 1_000_000_000))
            }
        }
    }

    /// "Refresh now": poll at once and read every watched folder in full.
    func refreshNow() {
        pendingFull.formUnion(watched.filter { $0.kind == .folder }.map(\.path))
        Task { await pollNow() }
    }

    func retry() {
        rebuildSession()
        Task { await pollNow() }
    }

    func pollNow() async {
        guard !polling else { return }
        guard let poller, session != nil else {
            updateSetupWithoutSession()
            return
        }
        polling = true
        defer { polling = false }
        let full = pendingFull.union(
            watched.filter { $0.kind == .folder && visible.contains("folder:\($0.path)") }.map(\.path))
        pendingFull = []
        let r = await poller.poll(watched, fullFolders: full)
        apply(r)
    }

    private func updateSetupWithoutSession() {
        if settings.repository.isEmpty {
            setup = .needsRepository
            icon = .error(SessionError.notConfigured.description)
        }
    }

    private func rebuildSession() {
        let old = session
        Task { await old?.stop() }
        session = nil
        poller = nil
        documents = [:]
        folders = [:]
        if settings.repository.isEmpty {
            setup = .needsRepository
            icon = .error(SessionError.notConfigured.description)
            return
        }
        let binary = settings.binaryPath.isEmpty ? nil : settings.binaryPath
        if Binary.find(setting: binary) == nil {
            setup = .notInstalled
            icon = .error(SessionError.notInstalled.description)
            return
        }
        let s = Session(
            config: Session.Config(
                binarySetting: binary, repository: settings.repository,
                clientVersion: Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String
                    ?? "dev"))
        session = s
        poller = Poller(caller: s)
        setup = .ready
        Task {
            await s.setElicitationHandler { method, params in
                guard method == "elicitation/create" else {
                    throw ToolError(code: "unsupported", message: "The app does not handle \(method).")
                }
                return await ApprovalDialog.ask(params["message"].string ?? "")
            }
        }
    }

    private func settingsChanged(from old: AppSettings) {
        settingsStore.settings = settings
        if old.repository != settings.repository || old.binaryPath != settings.binaryPath {
            if old.repository != settings.repository {
                reviewed.reset()
                activity = []
                reviewedVersion += 1
            }
            rebuildSession()
            Task { await pollNow() }
        }
        if old.startAtLogin != settings.startAtLogin { applyLoginItem() }
        if settings.notifications && !old.notifications { Notifier.requestPermission() }
    }

    // MARK: - Apply a poll

    private func apply(_ r: PollResult) {
        lastPoll = Date()
        icon = r.state
        if case .error(let text) = r.state {
            if text == SessionError.notInstalled.description {
                setup = .notInstalled
            } else if text == SessionError.notConfigured.description {
                setup = .needsRepository
            } else {
                setup = .problem(text)
            }
        } else {
            setup = .ready
        }
        if let l = r.limits {
            limits = l
            undoState = UndoState(l)
        }
        var notes: [(String, String, WatchedItem)] = []
        for item in watched {
            switch item.kind {
            case .document:
                guard let p = r.documents[item.path] else { continue }
                var st = documents[item.path] ?? DocumentState()
                st.problem = nil
                st.movedTo = nil
                switch p {
                case .loaded(let d):
                    st.document = d
                case .changed(let d):
                    st.document = d
                    st.changedAt = Date()
                    st.changedBy = d.changedBy
                    markChanged(item)
                    notes.append((L("%@ was updated by %@", item.label, d.changedBy), item.id, item))
                case .moved(let d):
                    st.document = d
                    st.movedTo = d.id
                    markChanged(item)
                case .deleted(let e):
                    if st.problem == nil && st.document != nil { markChanged(item) }
                    st.problem = L("This document was deleted.") + " " + e.message
                case .unavailable(let e):
                    st.problem = L("Not available") + ": " + e.message
                }
                documents[item.path] = st
                if case .loaded(let d) = p, item.title != d.title { renameTitle(item, d.title) }
            case .folder:
                guard let p = r.folders[item.path] else { continue }
                var st = folders[item.path] ?? FolderState()
                if let e = p.error {
                    st.problem = L("Not available") + ": " + e.message
                } else {
                    st.problem = nil
                    st.listing = p.listing ?? st.listing
                    st.newIDs.formUnion(p.newIDs)
                    st.changedIDs.formUnion(p.changedIDs)
                    for n in p.changedBy where !st.changedBy.contains(n) { st.changedBy.append(n) }
                    if p.hasChanges {
                        markChanged(item)
                        let text = p.changedBy.isEmpty
                            ? L("%@ was updated", item.label)
                            : L("%@ was updated by %@", item.label, p.changedBy.joined(separator: ", "))
                        notes.append((text, item.id, item))
                    }
                }
                folders[item.path] = st
            }
        }
        // Recent writes. On the first start, all listed writes count as reviewed.
        activity = r.activity
        if !reviewed.hasStarted && r.error == nil && session != nil {
            reviewed.mark(r.activity.map(\.commit))
            reviewed.hasStarted = true
            reviewedVersion += 1
        }
        let newMine = r.newWrites.filter { !reviewed.isReviewed($0) }
        if let path = UserDefaults.standard.string(forKey: "debugLogFile"), !path.isEmpty {
            let docs = documents.map { "\($0.key)=\($0.value.document?.version ?? $0.value.problem ?? "-")" }.sorted()
            let dirs = folders.map { "\($0.key)=\($0.value.listing?.hits.count ?? -1)/new\($0.value.newIDs.sorted())" }.sorted()
            Self.debugLine(
                path,
                "poll: state=\(r.state) docs=\(docs) folders=\(dirs) writes=\(activity.count) "
                    + "new=\(r.newWrites.count) unreviewed=\(unreviewedCount) dirty=\(dirty.sorted()) "
                    + "undo=\(undoState) error=\(r.error ?? "-") notices=\(r.notices)")
        }
        if settings.notifications {
            for n in notes { Notifier.post(title: "waymark", body: n.0, target: n.1) }
            if !newMine.isEmpty {
                Notifier.post(title: "waymark", body: Self.writesSentence(newMine.count, r.activity), target: "activity")
            }
        }
    }

    /// A line for the debug file (the default `debugLogFile`, for tests and support).
    static func debugLine(_ path: String, _ text: String) {
        let line = "\(Date().formatted(.iso8601)) \(text)\n"
        if let h = FileHandle(forWritingAtPath: path) {
            h.seekToEndOfFile()
            h.write(Data(line.utf8))
            try? h.close()
        } else {
            try? line.write(toFile: path, atomically: true, encoding: .utf8)
        }
    }

    static func writesSentence(_ count: Int, _ entries: [ActivityEntry]) -> String {
        if count == 1, let e = entries.first {
            return L("Your agent wrote “%@” (%@).", e.title, verb(e.verb))
        }
        return L("Your agents made %d writes.", count)
    }

    private func markChanged(_ item: WatchedItem) {
        let key = item.kind == .document ? "document:\(item.path)" : "folder:\(item.path)"
        if !visible.contains(key) { dirty.insert(item.id) }
    }

    private func renameTitle(_ item: WatchedItem, _ title: String) {
        var list = watchedStore.items
        if let i = list.firstIndex(where: { $0.id == item.id }) {
            list[i].title = title
            watchedStore.items = list
            watched = list
        }
    }

    // MARK: - Watched list

    func isWatched(_ kind: WatchedItem.Kind, _ path: String) -> Bool {
        watched.contains { $0.kind == kind && $0.path == path }
    }

    func watch(_ item: WatchedItem) {
        guard watchedStore.add(item) else { return }
        watched = watchedStore.items
        if item.kind == .folder { pendingFull.insert(item.path) }
        Task { await pollNow() }
    }

    func unwatch(_ item: WatchedItem) {
        watchedStore.remove(item)
        watched = watchedStore.items
        dirty.remove(item.id)
        let p = poller
        Task { await p?.forget(item) }
    }

    /// The document of a watched item moved: follow it to the new path.
    func follow(_ oldID: String, to newID: String) {
        if let item = watched.first(where: { $0.kind == .document && $0.path == oldID }) {
            watchedStore.rename(item, to: newID)
            watched = watchedStore.items
            documents[newID] = documents[oldID]
            documents[oldID] = nil
            documents[newID]?.movedTo = nil
        }
    }

    // MARK: - Windows

    func shown(_ key: String) {
        visible.insert(key)
        if let item = watched.first(where: { key == ($0.kind == .document ? "document:" : "folder:") + $0.path }) {
            dirty.remove(item.id)
        }
    }

    func hidden(_ key: String) {
        visible.remove(key)
        if key.hasPrefix("folder:") {
            let path = String(key.dropFirst("folder:".count))
            folders[path]?.newIDs = []
            folders[path]?.changedIDs = []
            folders[path]?.changedBy = []
        }
    }

    func open(_ item: WatchedItem) {
        switch item.kind {
        case .document: openWindow(id: "document", value: item.path)
        case .folder: openWindow(id: "folder", value: item.path)
        }
    }

    func openWindow(id: String, value: String? = nil) {
        NSApp.activate(ignoringOtherApps: true)
        if let value {
            openWindowAction?(id: id, value: value)
        } else {
            openWindowAction?(id: id)
        }
    }

    /// Open the window of a notification target ("document:…", "folder:…", "activity").
    func openTarget(_ target: String) {
        if target == "activity" { return openWindow(id: "activity") }
        if let item = watched.first(where: { $0.id == target }) { open(item) }
    }

    // MARK: - Reading

    func readFolder(_ path: String) async {
        guard let poller else { return }
        do {
            let l = try await poller.readFolder(path)
            var st = folders[path] ?? FolderState()
            st.listing = l
            st.problem = nil
            folders[path] = st
        } catch {
            var st = folders[path] ?? FolderState()
            st.problem = describe(error)
            folders[path] = st
        }
    }

    func show(_ id: String, version: String? = nil) async throws -> Document {
        guard let session else { throw SessionError.notConfigured }
        return try await session.show(id, version: version).value
    }

    func search(_ query: String, offset: Int) async throws -> SearchPage {
        guard let session else { throw SessionError.notConfigured }
        return try await session.search(query: query, level: 2, offset: offset).value
    }

    // MARK: - Recent writes

    func isReviewed(_ e: ActivityEntry) -> Bool {
        _ = reviewedVersion
        return reviewed.isReviewed(e.commit)
    }

    var unreviewedCount: Int {
        _ = reviewedVersion
        return activity.filter { !reviewed.isReviewed($0.commit) }.count
    }

    func markReviewed(_ commits: [String]) {
        reviewed.mark(commits)
        reviewedVersion += 1
    }

    func markUnreviewed(_ commit: String) {
        reviewed.unmark(commit)
        reviewedVersion += 1
    }

    func loadMore() async throws {
        guard let poller else { return }
        let older = try await poller.loadMore()
        activity = older
    }

    func change(of entry: ActivityEntry, id: String) async -> WriteChange {
        guard let session else { return WriteChange(id: id, shape: .unavailable(SessionError.notConfigured.description)) }
        return await RecentWrites.change(of: entry, id: id, caller: session)
    }

    func undoPreview(_ entry: ActivityEntry) async throws -> UndoResult {
        guard let session else { throw SessionError.notConfigured }
        return try await session.undo(commit: entry.commit, preview: true).value
    }

    /// Run the undo, then read `activity` again. The write and the undo commit count as reviewed.
    func undo(_ entry: ActivityEntry) async throws -> UndoResult {
        guard let session, let poller else { throw SessionError.notConfigured }
        let r = try await session.undo(commit: entry.commit, preview: false)
        var marks = [entry.commit]
        if let c = r.value.commit { marks.append(c) }
        markReviewed(marks)
        await poller.markUndone([entry.commit])
        activity = await poller.knownActivity
        await pollNow()
        if let l = r.limits { undoState = UndoState(l) }
        return r.value
    }

    // MARK: - Login item

    private func applyLoginItem() {
        do {
            if settings.startAtLogin {
                try SMAppService.mainApp.register()
            } else {
                try SMAppService.mainApp.unregister()
            }
        } catch {
            NSLog("waymark-bar: login item: \(error)")
        }
    }

    var loginItemStatus: String {
        switch SMAppService.mainApp.status {
        case .enabled: return L("On")
        case .requiresApproval: return L("Needs approval in System Settings, General, Login Items.")
        case .notRegistered: return L("Off")
        case .notFound: return L("Only available in the installed app.")
        @unknown default: return ""
        }
    }

    func describe(_ error: Error) -> String {
        if let e = error as? ToolError { return e.description }
        if let e = error as? SessionError { return e.description }
        if let e = error as? MCPError { return e.description }
        return error.localizedDescription
    }
}
