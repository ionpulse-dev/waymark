import XCTest
@testable import WaymarkBarCore

final class MCPClientTests: XCTestCase {
    // Review finding: a process that ends at once gives an error, not SIGPIPE, and no busy loop.
    func testAProcessThatEndsAtOnceFailsCleanly() async throws {
        let c = MCPClient(executable: URL(fileURLWithPath: "/usr/bin/true"), arguments: [], environment: [:])
        do {
            try await c.start(clientName: "test", clientVersion: "0", timeout: 10)
            XCTFail("expected an error")
        } catch let e as MCPError {
            if case .processExited = e {} else { XCTFail("\(e)") }
        }
        XCTAssertFalse(c.isRunning)
        // A second request after the exit fails at once.
        do {
            _ = try await c.request("tools/list", [:], timeout: 5)
            XCTFail("expected an error")
        } catch {}
    }

    func testAMissingBinaryCannotStart() async {
        let c = MCPClient(executable: URL(fileURLWithPath: "/nonexistent/waymark"), arguments: [], environment: [:])
        do {
            try await c.start(clientName: "test", clientVersion: "0", timeout: 5)
            XCTFail("expected an error")
        } catch let e as MCPError {
            if case .cannotStart = e {} else { XCTFail("\(e)") }
        } catch { XCTFail("\(error)") }
    }
}
