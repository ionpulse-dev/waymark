import XCTest
@testable import WaymarkBarCore

final class BinaryTests: XCTestCase {
    // U2
    func testFindOrder() {
        XCTAssertEqual(Binary.find(setting: nil, fileExists: { _ in true })?.path, "/opt/homebrew/bin/waymark")
        XCTAssertEqual(
            Binary.find(setting: "", fileExists: { $0 == "/usr/local/bin/waymark" })?.path, "/usr/local/bin/waymark")
        XCTAssertEqual(Binary.find(setting: "/x/waymark", fileExists: { _ in true })?.path, "/x/waymark")
        XCTAssertNil(Binary.find(setting: "/x/waymark", fileExists: { $0.hasPrefix("/opt") }))
        XCTAssertNil(Binary.find(setting: nil, fileExists: { _ in false }))
    }

    func testArgumentsHoldTheDenyList() {
        let args = Binary.arguments(repository: RepositorySetting(location: "/tmp/memory"))
        XCTAssertEqual(Array(args.prefix(2)), ["--path", "/tmp/memory"])
        let i = args.firstIndex(of: "--tools-denied")!
        let list = args[i + 1].split(separator: ",").map(String.init)
        XCTAssertFalse(list.contains("undo"))
        for t in ["add_document", "update_document", "delete_document", "move", "sync", "init"] {
            XCTAssertTrue(list.contains(t), t)
        }
        XCTAssertFalse(args.contains("--max-documents-opened"))
    }

    func testURLOrFolder() {
        XCTAssertTrue(RepositorySetting(location: "git@github.com:example/memory.git").isURL)
        XCTAssertTrue(RepositorySetting(location: "https://github.com/example/memory.git").isURL)
        XCTAssertFalse(RepositorySetting(location: "~/memory").isURL)
        let r = RepositorySetting(location: "git@github.com:e/m.git", cloneFolder: "~/m", memoryRepositoryID: "abc")
        XCTAssertEqual(r.arguments, ["--repo", "git@github.com:e/m.git", "--path", "~/m", "--memory-repository-id", "abc"])
    }

    func testEnvironmentAddsThePathsOfGit() {
        let env = Binary.environment(base: ["PATH": "/usr/bin"])
        XCTAssertTrue(env["PATH"]!.contains("/opt/homebrew/bin"))
        XCTAssertTrue(env["PATH"]!.hasPrefix("/usr/bin"))
    }

    func testUndoState() {
        var l = Limits()
        XCTAssertEqual(UndoState(l), .allowed)
        l.paused = true
        XCTAssertEqual(UndoState(l), .paused)
        l.access = "read-only"
        if case .notAllowed = UndoState(l) {} else { XCTFail() }
    }
}
