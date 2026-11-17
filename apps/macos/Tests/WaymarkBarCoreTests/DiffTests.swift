import XCTest
@testable import WaymarkBarCore

final class DiffTests: XCTestCase {
    // U1
    func testLinesMarkAddedRemovedAndKept() {
        let d = Diff.lines("a\nb\nc", "a\nx\nc\nd")
        XCTAssertEqual(d.map(\.kind), [.same, .removed, .added, .same, .added])
        XCTAssertEqual(d.map(\.text), ["a", "b", "x", "c", "d"])
        XCTAssertTrue(Diff.hasChanges(d))
    }

    func testSameTextHasNoChanges() {
        XCTAssertFalse(Diff.hasChanges(Diff.lines("a\nb", "a\nb")))
        XCTAssertEqual(Diff.lines("", "new").map(\.kind), [.added])
        XCTAssertEqual(Diff.lines("old", "").map(\.kind), [.removed])
    }

    func testChangedProperties() {
        var a = Document(id: "x", title: "Old", body: "", version: "1")
        var b = Document(id: "x", title: "New", body: "", version: "2")
        a.tags = ["one"]
        b.tags = ["one"]
        b.disputed = true
        b.disputeReason = "Wrong"
        let p = Diff.properties(a, b)
        XCTAssertEqual(p.map(\.name), ["Title", "Disputed"])
        XCTAssertEqual(p.first?.before, "Old")
        XCTAssertEqual(p.last?.after, "Wrong")
    }
}
