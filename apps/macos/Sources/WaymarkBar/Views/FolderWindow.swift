import SwiftUI
import WaymarkBarCore

/// The folder window (see "Folder window" in `requirements.md`).
struct FolderWindow: View {
    @Environment(AppModel.self) private var model
    @Environment(\.openWindow) private var openWindow
    let path: String

    private var state: FolderState? { model.folders[path] }
    private var item: WatchedItem? { model.watched.first { $0.kind == .folder && $0.path == path } }

    var body: some View {
        VStack(spacing: 0) {
            if let p = state?.problem {
                HStack { Text(p); Spacer() }.padding(8).background(Color.red.opacity(0.15))
            }
            if let names = state?.changedBy, !names.isEmpty {
                HStack { Text(L("Changed by %@", names.joined(separator: ", "))); Spacer() }
                    .padding(8).background(Color.blue.opacity(0.12))
            }
            if let listing = state?.listing {
                List(listing.hits) { hit in row(hit) }
                    .overlay {
                        if listing.hits.isEmpty { Text(L("This folder has no documents.")).foregroundStyle(.secondary) }
                    }
            } else if state?.problem == nil {
                ProgressView().frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                Spacer()
            }
        }
        .frame(minWidth: 380, minHeight: 300)
        .navigationTitle(path.isEmpty ? L("Memory repository") : path + "/")
        .toolbar {
            ToolbarItem {
                if let item {
                    Button(L("Stop watching")) { model.unwatch(item) }
                } else {
                    Button(L("Watch")) { model.watch(WatchedItem(kind: .folder, path: path)) }
                }
            }
        }
        .task { await model.readFolder(path) }
        .onAppear {
            model.openWindowAction = openWindow
            model.shown("folder:\(path)")
        }
        .onDisappear { model.hidden("folder:\(path)") }
    }

    private func row(_ hit: SearchHit) -> some View {
        let isNew = state?.newIDs.contains(hit.id) == true
        let isChanged = state?.changedIDs.contains(hit.id) == true
        return HStack(alignment: .firstTextBaseline) {
            VStack(alignment: .leading, spacing: 2) {
                HStack {
                    Text(hit.title).font(.headline)
                    if isNew { tag(L("New"), .green) } else if isChanged { tag(L("Changed"), .blue) }
                    if hit.status != "active" { tag(hit.status, .gray) }
                }
                if let d = hit.description { Text(d).foregroundStyle(.secondary).lineLimit(2) }
                Text(hit.id).font(.caption).foregroundStyle(.tertiary)
            }
            Spacer()
            Text(hit.updated).font(.caption).foregroundStyle(.secondary)
        }
        .contentShape(Rectangle())
        .onTapGesture { model.openWindow(id: "document", value: hit.id) }
    }

    private func tag(_ t: String, _ c: Color) -> some View {
        Text(t).font(.caption2).padding(.horizontal, 5).padding(.vertical, 1)
            .background(c.opacity(0.2), in: Capsule())
    }
}
