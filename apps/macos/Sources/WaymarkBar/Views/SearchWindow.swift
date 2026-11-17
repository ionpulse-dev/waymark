import SwiftUI
import WaymarkBarCore

/// The search window (see "Search window" in `requirements.md`).
struct SearchWindow: View {
    @Environment(AppModel.self) private var model
    @Environment(\.openWindow) private var openWindow
    @State private var query = ""
    @State private var hits: [SearchHit] = []
    @State private var total = 0
    @State private var error: String?
    @State private var loading = false

    var body: some View {
        VStack(spacing: 0) {
            TextField(L("Search the memory repository"), text: $query)
                .textFieldStyle(.roundedBorder)
                .padding(10)
            if let error {
                Text(error).foregroundStyle(.red).padding(.horizontal, 10).frame(maxWidth: .infinity, alignment: .leading)
            }
            List {
                ForEach(hits) { hit in row(hit) }
                if hits.count < total {
                    Button(L("More results (%d of %d)", hits.count, total)) { Task { await run(offset: hits.count) } }
                        .disabled(loading)
                }
            }
            .overlay {
                if hits.isEmpty && !query.isEmpty && !loading && error == nil {
                    Text(L("No documents found.")).foregroundStyle(.secondary)
                }
            }
            HStack {
                Button(L("Watch the whole memory repository")) {
                    model.watch(WatchedItem(kind: .folder, path: ""))
                }
                .disabled(model.isWatched(.folder, ""))
                Spacer()
                if loading { ProgressView().controlSize(.small) }
            }
            .padding(8)
        }
        .frame(minWidth: 420, minHeight: 320)
        .task(id: query) {
            try? await Task.sleep(nanoseconds: 300_000_000)
            if Task.isCancelled { return }
            await run(offset: 0)
        }
        .onAppear { model.openWindowAction = openWindow }
    }

    private func row(_ hit: SearchHit) -> some View {
        HStack(alignment: .top) {
            VStack(alignment: .leading, spacing: 2) {
                Text(hit.title).font(.headline)
                if let d = hit.description { Text(d).foregroundStyle(.secondary).lineLimit(2) }
                Text("\(hit.folder.isEmpty ? "/" : hit.folder + "/") · \(hit.updated)")
                    .font(.caption).foregroundStyle(.tertiary)
            }
            Spacer()
            Menu(L("Watch")) {
                Button(L("This document")) { model.watch(WatchedItem(kind: .document, path: hit.id, title: hit.title)) }
                    .disabled(model.isWatched(.document, hit.id))
                ForEach(folderChain(hit.folder), id: \.self) { f in
                    Button(L("Folder %@", f + "/")) { model.watch(WatchedItem(kind: .folder, path: f)) }
                        .disabled(model.isWatched(.folder, f))
                }
            }
            .fixedSize()
        }
        .contentShape(Rectangle())
        .onTapGesture { model.openWindow(id: "document", value: hit.id) }
    }

    /// `a/b/c` gives `a/b/c`, `a/b`, `a`.
    private func folderChain(_ folder: String) -> [String] {
        var out: [String] = []
        var f = folder
        while !f.isEmpty {
            out.append(f)
            f = DocumentID.folder(of: f)
        }
        return out
    }

    private func run(offset: Int) async {
        let q = query.trimmingCharacters(in: .whitespaces)
        guard !q.isEmpty else {
            hits = []
            total = 0
            error = nil
            return
        }
        loading = true
        defer { loading = false }
        do {
            let page = try await model.search(q, offset: offset)
            // A newer query or page may have started meanwhile.
            guard q == query.trimmingCharacters(in: .whitespaces), offset == (offset == 0 ? 0 : hits.count) else { return }
            if offset == 0 { hits = page.hits } else { hits += page.hits }
            total = page.total
            error = nil
        } catch {
            self.error = model.describe(error)
        }
    }
}
