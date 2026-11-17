import AppKit
import WaymarkBarCore

/// The native dialog for an `elicitation/create` request of `waymark` (see `write-approval` and
/// `plan.md`, step 9). Approve answers `accept` with `approve: true`, Decline answers `decline`.
enum ApprovalDialog {
    @MainActor
    static func present(_ message: String) -> JSON {
        NSApp.activate(ignoringOtherApps: true)
        let alert = NSAlert()
        alert.messageText = L("waymark asks for your approval")
        alert.informativeText = message.count > 4000 ? String(message.prefix(4000)) + "…" : message
        alert.alertStyle = .warning
        alert.addButton(withTitle: L("Approve"))
        alert.addButton(withTitle: L("Decline"))
        switch alert.runModal() {
        case .alertFirstButtonReturn:
            return ["action": "accept", "content": ["approve": true]]
        case .alertSecondButtonReturn:
            return ["action": "decline"]
        default:
            return ["action": "cancel"]
        }
    }

    static func ask(_ message: String) async -> JSON {
        await MainActor.run { present(message) }
    }
}
