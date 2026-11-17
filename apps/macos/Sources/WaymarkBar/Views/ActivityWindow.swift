import SwiftUI
import WaymarkBarCore

/// The recent writes window (see "Recent writes window" in `requirements.md`).
struct ActivityWindow: View {
    @Environment(AppModel.self) private var model
    @Environment(\.openWindow) private var openWindow
    @State private var selection: String?
    @State private var loadingMore = false
    @State private var moreError: String?

    private var selected: ActivityEntry? { model.activity.first { $0.commit == selection } }

    var body: some View {
        HSplitView {
            VStack(spacing: 0) {
                List(selection: $selection) {
                    ForEach(model.activity) { e in row(e).tag(e.commit) }
                    Button(loadingMore ? L("Loading…") : L("Load more")) { Task { await more() } }
                        .disabled(loadingMore)
                    if let moreError { Text(moreError).foregroundStyle(.red) }
                }
                .overlay {
                    if model.activity.isEmpty { Text(L("No writes under your name yet.")).foregroundStyle(.secondary) }
                }
                HStack {
                    Button(L("Mark all as reviewed")) { model.markReviewed(model.activity.map(\.commit)) }
                        .disabled(model.unreviewedCount == 0)
                    Spacer()
                    Text(L("%d not reviewed", model.unreviewedCount)).font(.caption).foregroundStyle(.secondary)
                }
                .padding(8)
            }
            .frame(minWidth: 320, idealWidth: 360)
            Group {
                if let e = selected {
                    WriteDetail(entry: e).id(e.commit)
                } else {
                    Text(L("Choose a write to see what changed.")).foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                }
            }
            .frame(minWidth: 420, maxWidth: .infinity, maxHeight: .infinity)
        }
        .frame(minWidth: 760, minHeight: 420)
        .onAppear {
            model.openWindowAction = openWindow
            model.shown("activity")
        }
        .onDisappear { model.hidden("activity") }
    }

    private func row(_ e: ActivityEntry) -> some View {
        let reviewed = model.isReviewed(e)
        return HStack(alignment: .top, spacing: 8) {
            Circle().fill(reviewed ? Color.clear : Color.accentColor).frame(width: 8, height: 8).padding(.top, 5)
            VStack(alignment: .leading, spacing: 2) {
                HStack {
                    Text(verb(e.verb)).bold()
                    Text(e.title).lineLimit(1)
                }
                HStack(spacing: 6) {
                    Text(time(e)).foregroundStyle(.secondary)
                    Text(e.folder.isEmpty ? "/" : e.folder + "/").foregroundStyle(.tertiary)
                    if e.undone { Text(L("Undone")).foregroundStyle(.orange) }
                    if e.laterChanges && !e.undone { Text(L("Changed later")).foregroundStyle(.secondary) }
                }
                .font(.caption)
            }
        }
    }

    private func time(_ e: ActivityEntry) -> String {
        guard let d = e.date else { return e.time }
        return d.formatted(date: .abbreviated, time: .shortened)
    }

    private func more() async {
        loadingMore = true
        defer { loadingMore = false }
        do {
            try await model.loadMore()
            moreError = nil
        } catch {
            moreError = model.describe(error)
        }
    }
}

func verb(_ v: String) -> String {
    switch v {
    case "add": return L("Added")
    case "update": return L("Updated")
    case "move": return L("Moved")
    case "delete": return L("Deleted")
    case "dispute": return L("Disputed")
    case "deprecate": return L("Deprecated")
    case "restore": return L("Restored")
    case "undo": return L("Undo")
    default: return v.capitalized
    }
}

/// The detail of one write: the change of each document, review, and Undo.
struct WriteDetail: View {
    @Environment(AppModel.self) private var model
    let entry: ActivityEntry
    @State private var chosen: String?
    @State private var change: WriteChange?
    @State private var showFull = false
    @State private var undoing = false

    private var id: String { chosen ?? entry.ids.first ?? "" }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            header.padding(12)
            Divider()
            ScrollView {
                VStack(alignment: .leading, spacing: 10) {
                    if entry.ids.count > 1 {
                        Picker(L("Document"), selection: Binding(get: { id }, set: { chosen = $0 })) {
                            ForEach(entry.ids, id: \.self) { Text($0).tag($0) }
                        }
                    }
                    detail
                }
                .padding(12)
                .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
        .task(id: id) {
            change = nil
            change = await model.change(of: entry, id: id)
        }
        .sheet(isPresented: $undoing) {
            UndoSheet(entry: entry) { model.openWindow(id: "document", value: $0) }.environment(model)
        }
    }

    private var header: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text("\(verb(entry.verb)) “\(entry.title)”").font(.title3).bold()
                Spacer()
            }
            Text("\(entry.date?.formatted(date: .abbreviated, time: .standard) ?? entry.time) · \(entry.commit)")
                .font(.caption).foregroundStyle(.secondary)
            if entry.undone {
                Text(L("Undone")).foregroundStyle(.orange)
            } else if entry.laterChanges, let first = entry.ids.first {
                HStack {
                    Text(L("Changed later."))
                    Button(L("Open the current document")) { model.openWindow(id: "document", value: first) }
                        .buttonStyle(.link)
                }
            }
            HStack {
                if model.isReviewed(entry) {
                    Button(L("Mark as not reviewed")) { model.markUnreviewed(entry.commit) }
                } else {
                    Button(L("Mark as reviewed")) { model.markReviewed([entry.commit]) }
                        .keyboardShortcut(.return, modifiers: .command)
                }
                Spacer()
                undoControl
            }
        }
    }

    @ViewBuilder private var undoControl: some View {
        if !entry.undone {
            switch model.undoState {
            case .allowed:
                Button(L("Undo…")) { undoing = true }
            case .paused:
                VStack(alignment: .trailing) {
                    Button(L("Undo…")) {}.disabled(true)
                    Text(L("Writes are paused.")).font(.caption).foregroundStyle(.secondary)
                }
            case .notAllowed(let why):
                Text(why).font(.caption).foregroundStyle(.secondary)
            }
        }
    }

    @ViewBuilder private var detail: some View {
        if let c = change {
            switch c.shape {
            case .added(let d):
                Text(L("New document")).font(.headline)
                MarkdownView(d.body, id: d.id) { model.openWindow(id: "document", value: $0) }
            case .changed(let before, let after, let lines, let props):
                Toggle(L("Show the full document"), isOn: $showFull)
                if showFull {
                    MarkdownView(after.body, id: after.id) { model.openWindow(id: "document", value: $0) }
                } else {
                    Text(L("Changes from version %@ to %@", before.version, after.version))
                        .font(.caption).foregroundStyle(.secondary)
                    DiffView(lines: lines, properties: props)
                }
            case .moved(let to):
                Text(L("Moved. The new path is %@.", to))
            case .deleted:
                Text(L("Deleted: “%@” in %@/. The app cannot show the text of a deleted document.", entry.title, entry.folder))
            case .unavailable(let why):
                Text(why).foregroundStyle(.secondary)
            }
        } else {
            ProgressView()
        }
    }
}
