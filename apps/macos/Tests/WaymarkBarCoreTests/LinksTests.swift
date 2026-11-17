import XCTest
@testable import WaymarkBarCore

final class LinksTests: XCTestCase {
    // U3
    func testWikiTargets() {
        XCTAssertEqual(Links.wikiTarget("ops/a"), "ops/a")
        XCTAssertEqual(Links.wikiTarget("ops/a|Label"), "ops/a")
        XCTAssertEqual(Links.wikiTarget("ops/a#Part"), "ops/a")
        XCTAssertEqual(Links.wikiTarget("ops/a.md"), "ops/a")
        XCTAssertEqual(Links.wikiLabel("ops/a|Label"), "Label")
    }

    func testRelativeTargets() {
        XCTAssertEqual(Links.relativeTarget("../b.md", from: "ops/deep/d"), "ops/b")
        XCTAssertEqual(Links.relativeTarget("c.md", from: "ops/deep/d"), "ops/deep/c")
        XCTAssertEqual(Links.relativeTarget("./c.md#part", from: "ops/d"), "ops/c")
        XCTAssertEqual(Links.relativeTarget("/top/e.md", from: "ops/d"), "top/e")
        XCTAssertNil(Links.relativeTarget("https://example.org/a.md", from: "ops/d"))
        XCTAssertNil(Links.relativeTarget("#part", from: "ops/d"))
        XCTAssertNil(Links.relativeTarget("../../x.md", from: "ops/d"))
        XCTAssertNil(Links.relativeTarget("image.png", from: "ops/d"))
    }

    func testRewriteMakesDocumentLinks() {
        let body = "See [[ops/a|A]] and [b](../b.md) and [web](https://example.org).\n```\n[[code]]\n```"
        let out = Links.rewrite(body, from: "ops/deep/d")
        XCTAssertTrue(out.contains("[A](waymark-doc:ops/a)"), out)
        XCTAssertTrue(out.contains("[b](waymark-doc:ops/b)"), out)
        XCTAssertTrue(out.contains("[web](https://example.org)"), out)
        XCTAssertTrue(out.contains("\n[[code]]\n"), out)
        XCTAssertEqual(Links.id(of: URL(string: "waymark-doc:ops/a")!), "ops/a")
    }

    func testEmbedsStayAsText() {
        XCTAssertEqual(Links.rewrite("![[pic.png]]", from: "a"), "![[pic.png]]")
    }
}
