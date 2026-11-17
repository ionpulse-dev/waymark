import SwiftUI
import WaymarkBarCore

/// The preview and confirm step of an undo (see "Recent writes window" in `requirements.md`).
struct UndoSheet: View {
    @Environment(AppModel.self) private var model
    @Environment(\.dismiss) private var dismiss
    let entry: ActivityEntry
    let onOpen: (String) -> Void
    @State private var preview: UndoResult?
    @State private var error: ToolErrorText?
    @State private var running = false
    @State private var done: UndoResult?

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(L("Undo \"%@\"?", entry.title)).font(.headline)
            if let done {
                Text(done.message)
                if done.pushed == false {
                    Text(L("The undo is saved on this Mac. It goes out with the next write or sync."))
                        .foregroundStyle(.secondary)
                }
            } else if let error {
                Text(error.message).foregroundStyle(.red)
                if let next = error.next { Text(next).foregroundStyle(.secondary) }
                if error.code == "stale_base" || entry.laterChanges, let id = entry.ids.first {
                    Button(L("Open the current document")) {
                        onOpen(id)
                        dismiss()
                    }
                }
            } else if let preview {
                Text(L("This is what changes:"))
                ForEach(preview.changes, id: \.self) { c in
                    Text("• \(action(c.action)): \(c.id)")
                }
                Text(L("The undo is a new commit under your name, pushed at once. It can be undone too."))
                    .font(.caption).foregroundStyle(.secondary)
            } else {
                HStack { ProgressView().controlSize(.small); Text(L("Preparing the preview…")) }
            }
            HStack {
                Spacer()
                if done != nil || error != nil {
                    Button(L("Close")) { dismiss() }.keyboardShortcut(.defaultAction)
                } else {
                    Button(L("Cancel")) { dismiss() }.keyboardShortcut(.cancelAction)
                    Button(L("Undo")) { Task { await run() } }
                        .keyboardShortcut(.defaultAction)
                        .disabled(preview == nil || running)
                }
            }
        }
        .padding(20)
        .frame(width: 460)
        .task { await loadPreview() }
    }

    private func action(_ a: String) -> String {
        switch a {
        case "removed": return L("removed")
        case "restored": return L("restored")
        case "reverted": return L("reverted")
        default: return a
        }
    }

    private func loadPreview() async {
        do {
            preview = try await model.undoPreview(entry)
        } catch {
            self.error = ToolErrorText(error, model)
        }
    }

    private func run() async {
        running = true
        defer { running = false }
        do {
            done = try await model.undo(entry)
        } catch {
            self.error = ToolErrorText(error, model)
        }
    }
}

/// A refusal of `undo`: the `message` and the `next` sentence.
struct ToolErrorText {
    var code: String
    var message: String
    var next: String?

    @MainActor
    init(_ error: Error, _ model: AppModel) {
        if let e = error as? ToolError {
            code = e.code
            message = e.message
            next = e.next
        } else {
            code = "error"
            message = model.describe(error)
            next = nil
        }
    }
}
