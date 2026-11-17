import XCTest
@testable import WaymarkBarCore

/// A fake `waymark` that answers from a script.
final class FakeCaller: ToolCaller, @unchecked Sendable {
    let lock = NSLock()
    var answers: [String: (JSON) throws -> JSON] = [:]
    var calls: [(String, JSON)] = []

    func call(_ tool: String, _ arguments: JSON) async throws -> JSON {
        let f: ((JSON) throws -> JSON)? = lock.withLock {
            calls.append((tool, arguments))
            return answers[tool]
        }
        guard let f else { throw ToolError(code: "not_found", message: "no answer for \(tool)") }
        return try f(arguments)
    }

    func count(_ tool: String) -> Int { lock.withLock { calls.filter { $0.0 == tool }.count } }
}

final class PollerTests: XCTestCase {
    func doc(_ version: String, notices: [String] = []) -> JSON {
        ["id": "ops/a", "title": "A", "body": "text", "version": .string(version), "notices": JSON(notices)]
    }

    func fake() -> FakeCaller {
        let f = FakeCaller()
        f.answers["activity"] = { _ in ["entries": [], "next_before": nil] }
        f.answers["updates"] = { _ in ["changes": []] }
        f.answers["search"] = { _ in
            ["results": [["id": "ops/a", "title": "A", "status": "active", "updated": "2026-11-17"]], "total_matches": 1]
        }
        return f
    }

    // U5
    func testNoChangeGivesNoEvent() async {
        let f = fake()
        f.answers["show"] = { _ in self.doc("v1") }
        let p = Poller(caller: f)
        let items = [WatchedItem(kind: .document, path: "ops/a"), WatchedItem(kind: .folder, path: "ops")]
        _ = await p.poll(items)
        let r = await p.poll(items)
        XCTAssertEqual(r.documents["ops/a"], .loaded(Document(json: doc("v1"))))
        XCTAssertEqual(r.folders["ops"]?.hasChanges, false)
        XCTAssertEqual(r.state, .normal)
        // The second poll did not read the folder in full: no change was named.
        XCTAssertEqual(f.count("search"), 1)
        XCTAssertEqual(f.count("updates"), 1)
    }

    func testANewVersionIsAChange() async {
        let f = fake()
        var version = "v1"
        f.answers["show"] = { _ in self.doc(version) }
        let p = Poller(caller: f)
        let items = [WatchedItem(kind: .document, path: "ops/a")]
        _ = await p.poll(items)
        version = "v2"
        let r = await p.poll(items)
        guard case .changed(let d) = r.documents["ops/a"] else { return XCTFail("\(r)") }
        XCTAssertEqual(d.version, "v2")
    }

    func testAnUpdateOfAFolderReadsItInFull() async {
        let f = fake()
        let p = Poller(caller: f)
        let items = [WatchedItem(kind: .folder, path: "ops")]
        _ = await p.poll(items)
        // The fake lists the change once: for a `since` before its time.
        let changedAt = Date().addingTimeInterval(1)
        f.answers["updates"] = { a in
            let since = Time.parse(a["since"].string ?? "") ?? .distantPast
            let list: JSON = since < changedAt ? [["id": "ops/sub/b", "kind": "new", "by": "Bob Roe <b@example.org>"]] : []
            return ["changes": list, "until": .string(Time.rfc3339(changedAt.addingTimeInterval(1)))]
        }
        f.answers["search"] = { _ in
            [
                "results": [
                    ["id": "ops/a", "title": "A", "status": "active", "updated": "2026-11-17"],
                    ["id": "ops/sub/b", "title": "B", "status": "active", "updated": "2026-11-18"],
                ], "total_matches": 2,
            ]
        }
        let r = await p.poll(items)
        XCTAssertEqual(r.folders["ops"]?.newIDs, ["ops/sub/b"])
        XCTAssertEqual(r.folders["ops"]?.changedBy, ["Bob Roe"])
        XCTAssertEqual(r.folders["ops"]?.listing?.hits.first?.id, "ops/sub/b")
        // The same change is not reported twice.
        let again = await p.poll(items)
        XCTAssertEqual(again.folders["ops"]?.hasChanges, false)
    }

    func testOwnWritesInAFolderReadItInFull() async {
        let f = fake()
        let p = Poller(caller: f)
        let items = [WatchedItem(kind: .folder, path: "ops")]
        _ = await p.poll(items)
        f.answers["activity"] = { _ in
            ["entries": [["commit": "c1", "verb": "add", "title": "C", "folder": "ops", "ids": ["ops/c"]]]]
        }
        f.answers["search"] = { _ in
            [
                "results": [
                    ["id": "ops/a", "title": "A", "status": "active", "updated": "2026-11-17"],
                    ["id": "ops/c", "title": "C", "status": "active", "updated": "2026-11-17"],
                ], "total_matches": 2,
            ]
        }
        let r = await p.poll(items)
        XCTAssertEqual(r.newWrites, ["c1"])
        XCTAssertEqual(r.folders["ops"]?.newIDs, ["ops/c"])
    }

    func testPagingReadsTheWholeFolder() async {
        let f = fake()
        f.answers["search"] = { a in
            let offset = a["offset"].int ?? 0
            let hits: [JSON] = (offset..<min(offset + 5, 12)).map {
                ["id": .string("ops/d\($0)"), "title": "D", "status": "active", "updated": "2026-11-17"]
            }
            return ["results": .array(hits), "total_matches": 12]
        }
        let p = Poller(caller: f)
        let r = await p.poll([WatchedItem(kind: .folder, path: "ops")])
        XCTAssertEqual(r.folders["ops"]?.listing?.hits.count, 12)
        XCTAssertEqual(f.count("search"), 3)
    }

    // U6
    func testANoticeMayBeBehindSetsBehind() async {
        let f = fake()
        f.answers["show"] = { _ in self.doc("v1", notices: ["Cannot reach the remote: x. The local copy may be behind."]) }
        let p = Poller(caller: f)
        let r = await p.poll([WatchedItem(kind: .document, path: "ops/a")])
        XCTAssertEqual(r.state, .behind)
        guard case .loaded = r.documents["ops/a"] else { return XCTFail() }
        // No new notice and no good fetch: still behind.
        f.answers["show"] = { _ in self.doc("v1") }
        f.answers["status"] = { _ in ["last_pull": nil] }
        let again = await p.poll([WatchedItem(kind: .document, path: "ops/a")])
        XCTAssertEqual(again.state, .behind)
        // A good fetch after it: normal.
        f.answers["status"] = { _ in ["last_pull": .string(Time.rfc3339(Date().addingTimeInterval(5)))] }
        let back = await p.poll([WatchedItem(kind: .document, path: "ops/a")])
        XCTAssertEqual(back.state, .normal)
    }

    func testAnErrorKeepsNothingAndSetsError() async {
        let f = fake()
        f.answers["activity"] = { _ in throw SessionError.notInstalled }
        let p = Poller(caller: f)
        let r = await p.poll([WatchedItem(kind: .document, path: "ops/a")])
        guard case .error = r.state else { return XCTFail() }
        XCTAssertNotNil(r.error)
    }

    func testADeletedDocument() async {
        let f = fake()
        f.answers["show"] = { _ in throw ToolError(code: "not_found", message: "gone") }
        let r = await Poller(caller: f).poll([WatchedItem(kind: .document, path: "ops/a")])
        guard case .deleted = r.documents["ops/a"] else { return XCTFail() }
    }

    func testRecentWritesPagesUntilTheKnownWrite() async throws {
        let f = FakeCaller()
        f.answers["activity"] = { a in
            if a["before"].isNull {
                return ["entries": [["commit": "n2"], ["commit": "n1"]], "next_before": "n1"]
            }
            return ["entries": [["commit": "k1"], ["commit": "k0"]], "next_before": "k0"]
        }
        let known = [ActivityEntry(commit: "k1", verb: "add", title: "", folder: "", ids: [])]
        let r = try await RecentWrites.refresh(f, known: known)
        XCTAssertEqual(r.new, ["n2", "n1", "k0"])
        XCTAssertEqual(r.entries.map(\.commit), ["n2", "n1", "k1", "k0"])
        XCTAssertEqual(f.count("activity"), 2)
    }
}

extension Document {
    init(json: JSON) { self.init(json) }
}
