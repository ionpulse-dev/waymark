import SwiftUI
import WaymarkBarCore

/// The document window (see "Document window" in `requirements.md`).
struct DocumentWindow: View {
    @Environment(AppModel.self) private var model
    @Environment(\.openWindow) private var openWindow
    let startID: String
    @State private var stack: [String] = []
    @State private var index = 0
    @State private var loaded: Document?
    @State private var error: String?
    @State private var now = Date()

    private var currentID: String { stack.indices.contains(index) ? stack[index] : startID }
    private var state: DocumentState? { model.documents[currentID] }
    private var watchedItem: WatchedItem? {
        model.watched.first { $0.kind == .document && $0.path == currentID }
    }
    /// A watched document comes from the poll. Others are read when opened.
    private var document: Document? { watchedItem != nil ? (state?.document ?? loaded) : loaded }

    var body: some View {
        VStack(spacing: 0) {
            banners
            ScrollView {
                if let d = document {
                    content(d).padding(16).frame(maxWidth: .infinity, alignment: .leading)
                } else if let e = error ?? state?.problem {
                    Text(e).foregroundStyle(.secondary).padding(24)
                } else {
                    ProgressView().padding(24)
                }
            }
        }
        .frame(minWidth: 420, minHeight: 300)
        .navigationTitle(document?.title ?? currentID)
        .toolbar {
            ToolbarItemGroup(placement: .navigation) {
                Button { go(index - 1) } label: { Image(systemName: "chevron.left") }
                    .help(L("Back")).disabled(index == 0)
                Button { go(index + 1) } label: { Image(systemName: "chevron.right") }
                    .help(L("Forward")).disabled(index >= stack.count - 1)
            }
            ToolbarItem {
                if let item = watchedItem {
                    Button(L("Stop watching")) { model.unwatch(item) }
                } else {
                    Button(L("Watch")) {
                        model.watch(WatchedItem(kind: .document, path: currentID, title: document?.title ?? ""))
                    }
                }
            }
        }
        .task(id: currentID) { await load() }
        .onAppear {
            if stack.isEmpty { stack = [startID] }
            model.openWindowAction = openWindow
            model.shown("document:\(currentID)")
        }
        .onDisappear { model.hidden("document:\(currentID)") }
        .onChange(of: currentID) { old, new in
            model.hidden("document:\(old)")
            model.shown("document:\(new)")
        }
        .onReceive(Timer.publish(every: 1, on: .main, in: .common).autoconnect()) { now = $0 }
    }

    @ViewBuilder private var banners: some View {
        if let s = state, let at = s.changedAt, now.timeIntervalSince(at) < 8 {
            banner(L("Updated just now by %@", s.changedBy ?? "?"), color: .blue)
        }
        if let to = state?.movedTo {
            HStack {
                Text(L("This document moved to %@.", to))
                Spacer()
                Button(L("Follow it")) {
                    model.follow(currentID, to: to)
                    open(to)
                }
            }
            .padding(8).background(Color.orange.opacity(0.2))
        }
        if let p = state?.problem, document != nil {
            banner(p, color: .red)
        }
        if let d = document {
            if d.disputed {
                banner(L("Disputed: %@", d.disputeReason ?? L("no reason given")), color: .orange)
            }
            if d.status == "deprecated" {
                HStack(spacing: 4) {
                    Text(L("Deprecated."))
                    if d.replacedBy.isEmpty {
                        Text(L("No replacement."))
                    } else {
                        Text(L("Replaced by"))
                        ForEach(d.replacedBy, id: \.self) { r in
                            Button(r) { open(r) }.buttonStyle(.link)
                        }
                    }
                    Spacer()
                }
                .padding(8).background(Color.yellow.opacity(0.25))
            }
        }
    }

    private func banner(_ text: String, color: Color) -> some View {
        HStack { Text(text); Spacer() }.padding(8).background(color.opacity(0.18))
    }

    private func content(_ d: Document) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text(d.title).font(.largeTitle).bold().textSelection(.enabled)
            if !d.description.isEmpty { Text(d.description).foregroundStyle(.secondary) }
            HStack(spacing: 12) {
                Label(d.status, systemImage: "circle.fill").labelStyle(.titleOnly)
                Text(L("Changed %@ by %@", d.updated, d.changedBy))
                if !d.tags.isEmpty { Text(d.tags.map { "#" + $0 }.joined(separator: " ")) }
            }
            .font(.caption).foregroundStyle(.secondary)
            Divider()
            MarkdownView(d.body, id: d.id) { open($0) }
        }
    }

    private func open(_ id: String) {
        stack = Array(stack.prefix(index + 1)) + [id]
        index = stack.count - 1
    }

    private func go(_ i: Int) {
        guard stack.indices.contains(i) else { return }
        index = i
    }

    private func load() async {
        error = nil
        if watchedItem != nil, state?.document != nil { return }
        do {
            loaded = try await model.show(currentID)
        } catch {
            loaded = nil
            self.error = model.describe(error)
        }
    }
}
