import SwiftUI
import WaymarkBarCore

/// Renders a document body (see `plan.md`, step 13). Basic Markdown: no tables.
struct MarkdownView: View {
    let body_: String
    let documentID: String
    let onOpen: (String) -> Void

    init(_ body: String, id: String, onOpen: @escaping (String) -> Void) {
        body_ = body
        documentID = id
        self.onOpen = onOpen
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            ForEach(Array(blocks.enumerated()), id: \.offset) { _, block in
                blockView(block)
            }
        }
        .textSelection(.enabled)
        .environment(\.openURL, OpenURLAction { url in
            if let id = Links.id(of: url) {
                onOpen(id)
                return .handled
            }
            return .systemAction
        })
    }

    enum Block {
        case heading(Int, String)
        case code(String)
        case text(String)
    }

    private var blocks: [Block] {
        let text = Links.rewrite(body_, from: documentID)
        var out: [Block] = []
        var para: [String] = []
        var code: [String]?
        func flush() {
            if !para.isEmpty { out.append(.text(para.joined(separator: "\n"))) }
            para = []
        }
        for line in text.components(separatedBy: "\n") {
            let t = line.trimmingCharacters(in: .whitespaces)
            if code != nil {
                if t.hasPrefix("```") || t.hasPrefix("~~~") {
                    out.append(.code(code!.joined(separator: "\n")))
                    code = nil
                } else {
                    code!.append(line)
                }
                continue
            }
            if t.hasPrefix("```") || t.hasPrefix("~~~") {
                flush()
                code = []
            } else if t.hasPrefix("#") {
                flush()
                let level = t.prefix(while: { $0 == "#" }).count
                out.append(.heading(level, t.dropFirst(level).trimmingCharacters(in: .whitespaces)))
            } else if t.isEmpty {
                flush()
            } else {
                para.append(line)
            }
        }
        if let code { out.append(.code(code.joined(separator: "\n"))) }
        flush()
        return out
    }

    @ViewBuilder private func blockView(_ b: Block) -> some View {
        switch b {
        case .heading(let level, let t):
            Text(inline(t)).font(level <= 1 ? .title : level == 2 ? .title2 : .title3).bold()
        case .code(let t):
            Text(t).font(.system(.body, design: .monospaced))
                .padding(8).frame(maxWidth: .infinity, alignment: .leading)
                .background(Color.secondary.opacity(0.1), in: RoundedRectangle(cornerRadius: 6))
        case .text(let t):
            Text(inline(t)).fixedSize(horizontal: false, vertical: true)
        }
    }

    private func inline(_ s: String) -> AttributedString {
        let opts = AttributedString.MarkdownParsingOptions(
            interpretedSyntax: .inlineOnlyPreservingWhitespace, failurePolicy: .returnPartiallyParsedIfPossible)
        return (try? AttributedString(markdown: s, options: opts)) ?? AttributedString(s)
    }
}
