import SwiftUI
import WaymarkBarCore

/// Shows a line diff and the changed properties.
struct DiffView: View {
    let lines: [Diff.Line]
    let properties: [Diff.Property]

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            if !properties.isEmpty {
                VStack(alignment: .leading, spacing: 4) {
                    ForEach(properties) { p in
                        HStack(alignment: .firstTextBaseline) {
                            Text(p.name).bold().frame(width: 90, alignment: .leading)
                            Text(p.before).strikethrough().foregroundStyle(.red)
                            Image(systemName: "arrow.right").foregroundStyle(.secondary)
                            Text(p.after).foregroundStyle(.green)
                        }
                    }
                }
                Divider()
            }
            if !Diff.hasChanges(lines) {
                Text(L("The text did not change.")).foregroundStyle(.secondary)
            }
            VStack(alignment: .leading, spacing: 0) {
                ForEach(lines) { l in
                    HStack(alignment: .top, spacing: 6) {
                        Text(mark(l.kind)).foregroundStyle(.secondary).frame(width: 12)
                        Text(l.text.isEmpty ? " " : l.text)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .font(.system(.body, design: .monospaced))
                    .padding(.horizontal, 4)
                    .background(color(l.kind))
                }
            }
            .textSelection(.enabled)
        }
    }

    private func mark(_ k: Diff.Kind) -> String {
        switch k {
        case .same: return " "
        case .added: return "+"
        case .removed: return "−"
        }
    }

    private func color(_ k: Diff.Kind) -> Color {
        switch k {
        case .same: return .clear
        case .added: return .green.opacity(0.18)
        case .removed: return .red.opacity(0.18)
        }
    }
}
