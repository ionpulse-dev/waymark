import XCTest
@testable import WaymarkBarCore

final class StoresTests: XCTestCase {
    private func defaults() -> UserDefaults {
        let name = "waymark-bar-test-\(UUID().uuidString)"
        let d = UserDefaults(suiteName: name)!
        d.removePersistentDomain(forName: name)
        return d
    }

    // U4
    func testReviewedSurvivesANewStore() {
        let d = defaults()
        ReviewedStore(defaults: d).mark(["abc1234", "def5678"])
        let again = ReviewedStore(defaults: d)
        XCTAssertTrue(again.isReviewed("abc1234"))
        XCTAssertFalse(again.isReviewed("0000000"))
    }

    func testReviewedKeepsTheLast1000() {
        let s = ReviewedStore(defaults: defaults())
        s.mark((0..<1005).map { "c\($0)" })
        XCTAssertEqual(s.commits.count, 1000)
        XCTAssertFalse(s.isReviewed("c0"))
        XCTAssertTrue(s.isReviewed("c1004"))
    }

    func testWatchedListAndSettingsSurvive() {
        let d = defaults()
        let w = WatchedStore(defaults: d)
        XCTAssertTrue(w.add(WatchedItem(kind: .folder, path: "ops")))
        XCTAssertFalse(w.add(WatchedItem(kind: .folder, path: "ops")))
        w.add(WatchedItem(kind: .document, path: "ops/a", title: "A"))
        XCTAssertEqual(WatchedStore(defaults: d).items.map(\.label), ["ops/", "A"])
        var s = AppSettings()
        s.pollInterval = 5
        s.repository.location = "~/memory"
        SettingsStore(defaults: d).settings = s
        let back = SettingsStore(defaults: d).settings
        XCTAssertEqual(back.repository.location, "~/memory")
        XCTAssertEqual(back.clampedInterval, 10)
    }
}
