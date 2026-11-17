import SwiftUI
import WaymarkBarCore

/// The menu bar icon and its states (see "Menu" in `requirements.md`).
struct MenuLabel: View {
    @Environment(AppModel.self) private var model
    @Environment(\.openWindow) private var openWindow

    var body: some View {
        let count = model.unreviewedCount
        HStack(spacing: 2) {
            Image(systemName: symbol)
            if count > 0 { Text("\(count)") }
        }
        .task { model.openWindowAction = openWindow }
        .accessibilityLabel(L("waymark"))
    }

    private var symbol: String {
        switch model.icon {
        case .normal: return model.unreviewedCount > 0 ? "bookmark.fill" : "bookmark"
        case .behind: return "bookmark.slash"
        case .error: return "exclamationmark.triangle"
        }
    }
}

/// The menu.
struct MenuView: View {
    @Environment(AppModel.self) private var model
    @Environment(\.openWindow) private var openWindow

    var body: some View {
        Group {
            stateLines
            Divider()
            Button(L("Search…")) { open("search") }
                .keyboardShortcut("f")
                .disabled(model.session == nil)
            Button(writesTitle) { open("activity") }
                .disabled(model.session == nil)
            Divider()
            if model.watched.isEmpty {
                Text(L("Nothing watched yet. Use Search to watch a document or a folder."))
            } else {
                ForEach(model.watched) { item in
                    Button((model.dirty.contains(item.id) ? "● " : "    ") + item.label) {
                        model.openWindowAction = openWindow
                        model.open(item)
                    }
                }
            }
            Divider()
            Button(L("Refresh now")) { model.refreshNow() }
                .keyboardShortcut("r")
                .disabled(model.session == nil || model.polling)
            SettingsLink { Text(L("Settings…")) }
                .keyboardShortcut(",")
            Button(L("Quit")) { NSApp.terminate(nil) }
                .keyboardShortcut("q")
        }
        .onAppear { model.openWindowAction = openWindow }
    }

    @ViewBuilder private var stateLines: some View {
        switch model.setup {
        case .needsRepository:
            Text(L("No memory repository is set. Choose one in Settings."))
        case .notInstalled:
            Text(L("waymark is not installed. Install it with Homebrew, then choose Retry."))
            Button(L("Retry")) { model.retry() }
        case .problem(let text):
            Text(text).lineLimit(4)
            Button(L("Retry")) { model.retry() }
        case .ready:
            if case .behind = model.icon {
                Text(L("Behind: the remote cannot be reached. The windows show the last content."))
            } else if let t = model.lastPoll {
                Text(L("Updated %@", t.formatted(date: .omitted, time: .shortened)))
            } else {
                Text(L("Starting…"))
            }
        }
    }

    private var writesTitle: String {
        let n = model.unreviewedCount
        return n > 0 ? L("Recent writes (%d not reviewed)", n) : L("Recent writes")
    }

    private func open(_ id: String) {
        model.openWindowAction = openWindow
        model.openWindow(id: id)
    }
}
