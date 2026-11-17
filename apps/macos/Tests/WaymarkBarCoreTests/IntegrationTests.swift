import Foundation
import XCTest
@testable import WaymarkBarCore

/// A real `waymark` against a temporary memory repository (see `test.md`, A1 to A19).
final class IntegrationTests: XCTestCase {
    var f: Fixture!
    var session: Session!

    override func setUp() async throws {
        f = try Fixture()
        try await f.initialize()
        session = f.appSession()
    }

    override func tearDown() async throws {
        await session?.stop()
        session = nil
        f = nil
    }

    // A1
    func testUndoIsTheOnlyWriteTool() async throws {
        _ = try await session.status()
        let tools = await session.tools
        XCTAssertFalse(tools.isEmpty)
        let writes = tools.filter { !$0.value }.map(\.key)
        XCTAssertEqual(writes, ["undo"], "\(tools)")
        for t in ["search", "show", "activity", "history", "updates", "status"] {
            XCTAssertEqual(tools[t], true, t)
        }
    }

    // A1: a newer waymark with a write tool that the app does not know is switched off too.
    func testEveryWriteToolOfWaymarkIsInTheDenyList() async throws {
        let ag = try await f.agent(.a)
        let list = try await ag.client.request("tools/list")
        let writes = list["tools"].array.filter { $0["annotations"]["readOnlyHint"].bool == false }.compactMap {
            $0["name"].string
        }
        for t in writes where t != "undo" {
            XCTAssertTrue(Binary.deniedTools.contains(t), "the app does not switch off \(t)")
        }
    }

    // A2
    func testAMissingBinaryIsNotInstalled() async throws {
        let s = Session(
            config: Session.Config(binarySetting: "/nonexistent/waymark", repository: RepositorySetting(location: f.a.path)))
        do {
            _ = try await s.status()
            XCTFail("expected an error")
        } catch let e as SessionError {
            XCTAssertEqual(e, .notInstalled)
        }
        let r = await Poller(caller: s).poll([])
        guard case .error(let text) = r.state else { return XCTFail() }
        XCTAssertTrue(text.contains("not installed"), text)
    }

    // A6
    func testAPushOfAnotherPersonShowsAtTheNextPoll() async throws {
        let a = try await f.agent(.a)
        try await a.add("ops", "Deploy process", body: "Step one.")
        let poller = Poller(caller: session)
        let items = [WatchedItem(kind: .document, path: "ops/deploy-process")]
        let first = await poller.poll(items)
        guard case .loaded = first.documents["ops/deploy-process"] else { return XCTFail("\(first)") }
        let b = try await f.agent(.b)
        try await b.update("ops/deploy-process", body: "Step one.\n\nStep two.")
        let r = await poller.poll(items)
        guard case .changed(let d) = r.documents["ops/deploy-process"] else { return XCTFail("\(r)") }
        XCTAssertTrue(d.body.contains("Step two."))
        XCTAssertEqual(d.changedBy, "Bob Roe")
        XCTAssertEqual(r.state, .normal)
    }

    // A7
    func testANewDocumentInASubfolderIsNew() async throws {
        let a = try await f.agent(.a)
        try await a.add("ops", "First")
        let poller = Poller(caller: session)
        let items = [WatchedItem(kind: .folder, path: "ops")]
        let first = await poller.poll(items)
        XCTAssertEqual(first.folders["ops"]?.listing?.hits.map(\.id), ["ops/first"])
        let b = try await f.agent(.b)
        try await b.add("ops/deep", "Second")
        let r = await poller.poll(items)
        XCTAssertEqual(r.folders["ops"]?.newIDs, ["ops/deep/second"], "\(r)")
        XCTAssertEqual(r.folders["ops"]?.changedBy, ["Bob Roe"])
        XCTAssertEqual(Set(r.folders["ops"]?.listing?.hits.map(\.id) ?? []), ["ops/first", "ops/deep/second"])
    }

    // A8
    func testNoChangeGivesNoEvent() async throws {
        let a = try await f.agent(.a)
        try await a.add("ops", "Calm")
        let poller = Poller(caller: session)
        let items = [WatchedItem(kind: .document, path: "ops/calm"), WatchedItem(kind: .folder, path: "ops")]
        _ = await poller.poll(items)
        let r = await poller.poll(items)
        guard case .loaded = r.documents["ops/calm"] else { return XCTFail("\(r)") }
        XCTAssertEqual(r.folders["ops"]?.hasChanges, false)
        XCTAssertTrue(r.newWrites.isEmpty)
    }

    // A9
    func testADeprecatedDocumentNamesItsReplacement() async throws {
        let a = try await f.agent(.a)
        try await a.add("ops", "Old way")
        _ = try await a.call(
            "add_document",
            [
                "folder": "ops", "title": "New way", "description": "The new way", "body": "New.",
                "supersedes": "ops/old-way",
            ])
        let d = try await session.show("ops/old-way").value
        XCTAssertEqual(d.status, "deprecated")
        XCTAssertEqual(d.replacedBy, ["ops/new-way"])
    }

    // A10
    func testNoNetworkKeepsTheContentAndShowsBehind() async throws {
        let a = try await f.agent(.a)
        try await a.add("ops", "Offline")
        let poller = Poller(caller: session)
        let items = [WatchedItem(kind: .document, path: "ops/offline")]
        _ = await poller.poll(items)
        try FileManager.default.moveItem(at: f.remote, to: f.root.appendingPathComponent("gone.git"))
        let r = await poller.poll(items)
        XCTAssertEqual(r.state, .behind, "\(r.notices)")
        guard case .loaded(let d) = r.documents["ops/offline"] else { return XCTFail("\(r)") }
        XCTAssertEqual(d.title, "Offline")
        // waymark gives the notice once per session. The state stays until a fetch works again.
        try await Task.sleep(nanoseconds: 1_100_000_000)
        let again = await poller.poll(items)
        XCTAssertEqual(again.state, .behind, "\(again.notices)")
        try FileManager.default.moveItem(at: f.root.appendingPathComponent("gone.git"), to: f.remote)
        let back = await poller.poll(items)
        XCTAssertEqual(back.state, .normal, "\(back.notices)")
    }

    // A11
    func testANewWriteOfThePersonIsListedAndOthersAreNot() async throws {
        let store = ReviewedStore(defaults: UserDefaults(suiteName: "bar-it-\(UUID().uuidString)")!)
        let poller = Poller(caller: session)
        _ = await poller.poll([])
        let a = try await f.agent(.a)
        try await a.add("ops", "Mine")
        let b = try await f.agent(.b)
        try await b.add("ops", "Theirs")
        let r = await poller.poll([])
        XCTAssertEqual(r.newWrites.count, 1, "\(r.activity)")
        let mine = r.activity.first { r.newWrites.contains($0.commit) }
        XCTAssertEqual(mine?.title, "Mine")
        XCTAssertFalse(r.activity.contains { $0.title == "Theirs" })
        let unreviewed = r.activity.filter { !store.isReviewed($0.commit) && r.newWrites.contains($0.commit) }
        XCTAssertEqual(unreviewed.count, 1)
    }

    // A12
    func testTheChangeOfAnUpdateIsADiff() async throws {
        let a = try await f.agent(.a)
        try await a.add("ops", "Diffed", body: "Line one.")
        try await a.update("ops/diffed", body: "Line one.\n\nLine two.")
        let entries = try await session.activity().value.entries
        let update = try XCTUnwrap(entries.first { $0.verb == "update" })
        let c = await RecentWrites.change(of: update, id: "ops/diffed", caller: session)
        guard case .changed(let before, let after, let lines, _) = c.shape else { return XCTFail("\(c)") }
        XCTAssertEqual(before.body, "Line one.")
        XCTAssertTrue(after.body.contains("Line two."))
        XCTAssertTrue(lines.contains { $0.kind == .added && $0.text == "Line two." })
        let add = try XCTUnwrap(entries.first { $0.verb == "add" })
        let ca = await RecentWrites.change(of: add, id: "ops/diffed", caller: session)
        guard case .added(let d) = ca.shape else { return XCTFail("\(ca)") }
        XCTAssertEqual(d.body, "Line one.")
    }

    // A13, A14
    func testUndoPreviewThenUndo() async throws {
        let a = try await f.agent(.a)
        try await a.add("ops", "Undo me", body: "One.")
        try await a.update("ops/undo-me", body: "Two.")
        let listed = try await session.activity().value.entries
        let update = try XCTUnwrap(listed.first { $0.verb == "update" })
        let head = try f.remoteHead()
        let preview = try await session.undo(commit: update.commit, preview: true).value
        XCTAssertTrue(preview.preview)
        XCTAssertEqual(preview.changes.first?.id, "ops/undo-me")
        XCTAssertEqual(try f.remoteHead(), head, "a preview writes nothing")
        let done = try await session.undo(commit: update.commit, preview: false).value
        XCTAssertEqual(done.pushed, true)
        XCTAssertNotEqual(try f.remoteHead(), head)
        let author = try f.git(f.remote, "log", "-1", "--format=%an", "main").trimmingCharacters(in: .whitespacesAndNewlines)
        XCTAssertEqual(author, "Anna Example")
        let poller = Poller(caller: session)
        let r = await poller.poll([])
        XCTAssertEqual(r.activity.first?.verb, "undo")
        XCTAssertEqual(r.activity.first { $0.commit == update.commit }?.undone, true)
        let body = try await session.show("ops/undo-me").value.body
        XCTAssertEqual(body, "One.")
    }

    // A15
    func testUndoRefusesAfterALaterChangeOfAnother() async throws {
        let a = try await f.agent(.a)
        try await a.add("ops", "Contested", body: "One.")
        try await a.update("ops/contested", body: "Two.")
        let listed = try await session.activity().value.entries
        let update = try XCTUnwrap(listed.first { $0.verb == "update" })
        let b = try await f.agent(.b)
        try await b.update("ops/contested", body: "Three.")
        let head = try f.remoteHead()
        do {
            _ = try await session.undo(commit: update.commit, preview: false)
            XCTFail("expected a refusal")
        } catch let e as ToolError {
            XCTAssertFalse(e.message.isEmpty)
            XCTAssertNotNil(e.next)
        }
        XCTAssertEqual(try f.remoteHead(), head)
    }

    // A16
    func testApprovalAsksTheAppAndRunsOnlyAfterAYes() async throws {
        let a = try await f.agent(.a)
        try await a.add("ops", "Approved", body: "One.")
        try await a.update("ops/approved", body: "Two.")
        try f.personalConfig("[approval]\nrequired_for = [\"undo\"]\n")
        let s = f.appSession()
        defer { Task { await s.stop() } }
        let asked = Counter()
        let answer = Counter()
        await s.setElicitationHandler { method, params in
            XCTAssertEqual(method, "elicitation/create")
            XCTAssertNotNil(params["message"].string)
            asked.add()
            return answer.value == 0 ? ["action": "decline"] : ["action": "accept", "content": ["approve": true]]
        }
        let listed = try await s.activity().value.entries
        let update = try XCTUnwrap(listed.first { $0.verb == "update" })
        let head = try f.remoteHead()
        do {
            _ = try await s.undo(commit: update.commit, preview: false)
            XCTFail("expected approval_denied")
        } catch let e as ToolError {
            XCTAssertEqual(e.code, "approval_denied")
        }
        XCTAssertEqual(asked.value, 1)
        XCTAssertEqual(try f.remoteHead(), head)
        answer.add()
        let done = try await s.undo(commit: update.commit, preview: false).value
        XCTAssertEqual(asked.value, 2)
        XCTAssertEqual(done.pushed, true)
    }

    // A17
    func testReadOnlyInTheSharedSettingsHidesUndo() async throws {
        let settings = f.b.appendingPathComponent("agent-memory/.waymark/settings.toml")
        try f.git(f.b, "pull", "-q")
        var text = try String(contentsOf: settings, encoding: .utf8)
        if text.contains("[limits]") {
            text = text.replacingOccurrences(of: "[limits]\n", with: "[limits]\naccess = \"read-only\"\n")
        } else {
            text += "\n[limits]\naccess = \"read-only\"\n"
        }
        try text.write(to: settings, atomically: true, encoding: .utf8)
        try f.git(f.b, "commit", "-q", "-am", "read only")
        try f.git(f.b, "push", "-q")
        // The first call pulls the new settings. A result carries the limits read at its start, so the
        // next poll shows them.
        _ = try await session.activity()
        let r = try await session.activity()
        let limits = try XCTUnwrap(r.limits)
        guard case .notAllowed(let why) = UndoState(limits) else { return XCTFail("\(limits)") }
        XCTAssertTrue(why.contains("read-only"))
    }

    // A18
    func testTheAppDoesNotMoveTheStoredTimeOfTheAgents() async throws {
        let a = try await f.agent(.a)
        _ = try await a.call("updates", [:])  // the agent's own last call
        // Commit times have whole seconds. Bob's change must be in a later second than the stored time.
        try await Task.sleep(nanoseconds: 1_500_000_000)
        let b = try await f.agent(.b)
        try await b.add("ops", "From Bob")
        let poller = Poller(caller: session)
        _ = await poller.poll([WatchedItem(kind: .folder, path: "ops")])
        _ = await poller.poll([WatchedItem(kind: .folder, path: "ops")])
        let u = try await a.call("updates", [:])
        XCTAssertTrue(u["changes"].array.contains { $0["id"].string == "ops/from-bob" }, "\(u)")
    }

    // A19
    func testTheReadBudgetStartsANewSession() async throws {
        let a = try await f.agent(.a)
        for i in 0..<5 { try await a.add("ops", "Budget \(i)") }
        try f.personalConfig("[limits]\nmax_documents_opened = 2\n")
        let s = f.appSession()
        defer { Task { await s.stop() } }
        // The same document again costs nothing: waymark counts each ID once.
        for _ in 0..<3 { _ = try await s.show("ops/budget-0") }
        var starts = await s.starts
        XCTAssertEqual(starts, 1)
        for i in 0..<5 {
            let d = try await s.show("ops/budget-\(i)").value
            XCTAssertEqual(d.title, "Budget \(i)")
        }
        starts = await s.starts
        XCTAssertGreaterThanOrEqual(starts, 3)
    }

    // Review finding: calls that start together share one process.
    func testParallelCallsStartOneProcess() async throws {
        let s = f.appSession()
        defer { Task { await s.stop() } }
        try await withThrowingTaskGroup(of: Void.self) { g in
            for _ in 0..<6 { g.addTask { _ = try await s.status() } }
            try await g.waitForAll()
        }
        let starts = await s.starts
        XCTAssertEqual(starts, 1)
    }

    // Review finding: a restart (the read budget) does not stop an undo that waits for approval.
    func testARestartDoesNotBreakAnUndoInFlight() async throws {
        let a = try await f.agent(.a)
        try await a.add("ops", "Slow", body: "One.")
        try await a.update("ops/slow", body: "Two.")
        try f.personalConfig("[approval]\nrequired_for = [\"undo\"]\n")
        let s = f.appSession()
        defer { Task { await s.stop() } }
        await s.setElicitationHandler { _, _ in
            try await Task.sleep(nanoseconds: 1_500_000_000)
            return ["action": "accept", "content": ["approve": true]]
        }
        let listed = try await s.activity().value.entries
        let update = try XCTUnwrap(listed.first { $0.verb == "update" })
        async let undone = s.undo(commit: update.commit, preview: false)
        try await Task.sleep(nanoseconds: 500_000_000)
        try await s.restart()
        let r = try await undone
        XCTAssertEqual(r.value.pushed, true)
        let body = try await s.show("ops/slow").value.body
        XCTAssertEqual(body, "One.")
    }

    // The search window: paging with the small max_results.
    func testSearchPages() async throws {
        let a = try await f.agent(.a)
        for i in 0..<7 { try await a.add("ops", "Paged document \(i)") }
        let first = try await session.search(query: "paged", level: 2).value
        XCTAssertEqual(first.total, 7)
        XCTAssertEqual(first.hits.count, 5)
        let next = try await session.search(query: "paged", level: 2, offset: 5).value
        XCTAssertEqual(next.hits.count, 2)
    }
}

final class Counter: @unchecked Sendable {
    private let lock = NSLock()
    private var n = 0
    var value: Int { lock.withLock { n } }
    func add() { lock.withLock { n += 1 } }
}
