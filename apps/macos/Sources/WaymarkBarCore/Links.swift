import Foundation

/// Turns links in a document body into document IDs (see `plan.md`, step 13).
public enum Links {
    /// The URL scheme of a link to a document. `waymark-doc:ops/deploy`.
    public static let scheme = "waymark-doc"

    /// The ID that a wiki link target names: `ops/a`, `ops/a|Label`, `ops/a#Part`, `ops/a.md`.
    public static func wikiTarget(_ inner: String) -> String {
        var t = inner
        if let bar = t.firstIndex(of: "|") { t = String(t[..<bar]) }
        if let hash = t.firstIndex(of: "#") { t = String(t[..<hash]) }
        t = t.trimmingCharacters(in: .whitespaces)
        if t.hasSuffix(".md") { t.removeLast(3) }
        while t.hasPrefix("/") { t.removeFirst() }
        return t
    }

    /// The label of a wiki link: the part after `|`, else the target.
    public static func wikiLabel(_ inner: String) -> String {
        if let bar = inner.firstIndex(of: "|") {
            return inner[inner.index(after: bar)...].trimmingCharacters(in: .whitespaces)
        }
        return inner.trimmingCharacters(in: .whitespaces)
    }

    /// The ID that a relative Markdown link names, seen from the document `from`. Nil for an external
    /// link, an anchor, or a link that leaves the memory repository.
    public static func relativeTarget(_ href: String, from: String) -> String? {
        var h = href.trimmingCharacters(in: .whitespaces)
        if h.isEmpty || h.hasPrefix("#") || h.contains(":") || h.hasPrefix("//") { return nil }
        if let hash = h.firstIndex(of: "#") { h = String(h[..<hash]) }
        if let q = h.firstIndex(of: "?") { h = String(h[..<q]) }
        h = h.removingPercentEncoding ?? h
        guard h.hasSuffix(".md") else { return nil }
        h.removeLast(3)
        var parts: [String] = h.hasPrefix("/") ? [] : DocumentID.folder(of: from).split(separator: "/").map(String.init)
        for p in h.split(separator: "/").map(String.init) {
            switch p {
            case ".", "": continue
            case "..":
                if parts.isEmpty { return nil }
                parts.removeLast()
            default: parts.append(p)
            }
        }
        return parts.isEmpty ? nil : parts.joined(separator: "/")
    }

    /// The body with wiki links and relative links turned into `waymark-doc:` links, for
    /// `AttributedString(markdown:)`. Code blocks stay as they are.
    public static func rewrite(_ body: String, from: String) -> String {
        var out: [String] = []
        var inFence = false
        for line in body.components(separatedBy: "\n") {
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            if trimmed.hasPrefix("```") || trimmed.hasPrefix("~~~") {
                inFence.toggle()
                out.append(line)
                continue
            }
            out.append(inFence ? line : rewriteLine(line, from: from))
        }
        return out.joined(separator: "\n")
    }

    public static func url(for id: String) -> URL? {
        var c = URLComponents()
        c.scheme = scheme
        c.path = id
        return c.url
    }

    /// The document ID of a `waymark-doc:` URL.
    public static func id(of url: URL) -> String? {
        guard url.scheme == scheme else { return nil }
        let c = URLComponents(url: url, resolvingAgainstBaseURL: false)
        return c?.path.removingPercentEncoding ?? c?.path
    }

    private static func rewriteLine(_ line: String, from: String) -> String {
        // Wiki links: [[target]] and [[target|label]]. An embed ![[...]] stays as text.
        var s = ""
        var rest = Substring(line)
        while let open = rest.range(of: "[[") {
            guard let close = rest[open.upperBound...].range(of: "]]") else { break }
            let before = rest[..<open.lowerBound]
            let inner = String(rest[open.upperBound..<close.lowerBound])
            if before.hasSuffix("!") || inner.isEmpty {
                s += rest[..<close.upperBound]
            } else {
                let target = wikiTarget(inner)
                let label = wikiLabel(inner).replacingOccurrences(of: "]", with: "\\]")
                if let u = url(for: target) {
                    s += before + "[\(label)](\(u.absoluteString))"
                } else {
                    s += rest[..<close.upperBound]
                }
            }
            rest = rest[close.upperBound...]
        }
        s += rest
        // Relative Markdown links: [text](path.md).
        var result = ""
        var r = Substring(s)
        while let mid = r.range(of: "](") {
            guard let end = r[mid.upperBound...].firstIndex(of: ")") else { break }
            let href = String(r[mid.upperBound..<end])
            result += r[..<mid.upperBound]
            if let id = relativeTarget(href, from: from), let u = url(for: id) {
                result += u.absoluteString
            } else {
                result += href
            }
            result += ")"
            r = r[r.index(after: end)...]
        }
        result += r
        return result
    }
}
