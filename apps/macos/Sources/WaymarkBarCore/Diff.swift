import Foundation

/// A line diff for display only (see `plan.md`, step 7). It does not parse or check documents.
public enum Diff {
    public enum Kind: Equatable, Sendable { case same, added, removed }

    public struct Line: Equatable, Sendable, Identifiable {
        public var id: Int
        public var kind: Kind
        public var text: String
    }

    public struct Property: Equatable, Sendable, Identifiable {
        public var id: String { name }
        public var name: String
        public var before: String
        public var after: String
    }

    /// The lines of `after` compared with `before` (longest common subsequence).
    public static func lines(_ before: String, _ after: String) -> [Line] {
        let a = split(before), b = split(after)
        let n = a.count, m = b.count
        // Very long texts: no table of n*m, show all removed then all added.
        if n * m > 4_000_000 {
            return number(a.map { (Kind.removed, $0) } + b.map { (Kind.added, $0) })
        }
        var t = [[Int]](repeating: [Int](repeating: 0, count: m + 1), count: n + 1)
        if n > 0 && m > 0 {
            for i in stride(from: n - 1, through: 0, by: -1) {
                for j in stride(from: m - 1, through: 0, by: -1) {
                    t[i][j] = a[i] == b[j] ? t[i + 1][j + 1] + 1 : max(t[i + 1][j], t[i][j + 1])
                }
            }
        }
        var out: [(Kind, String)] = []
        var i = 0, j = 0
        while i < n && j < m {
            if a[i] == b[j] {
                out.append((.same, a[i])); i += 1; j += 1
            } else if t[i + 1][j] >= t[i][j + 1] {
                out.append((.removed, a[i])); i += 1
            } else {
                out.append((.added, b[j])); j += 1
            }
        }
        while i < n { out.append((.removed, a[i])); i += 1 }
        while j < m { out.append((.added, b[j])); j += 1 }
        return number(out)
    }

    /// The properties that differ between two versions.
    public static func properties(_ before: Document, _ after: Document) -> [Property] {
        zip(before.properties, after.properties).compactMap { b, a in
            b.1 == a.1 ? nil : Property(name: a.0, before: b.1, after: a.1)
        }
    }

    /// Does the diff change anything?
    public static func hasChanges(_ lines: [Line]) -> Bool { lines.contains { $0.kind != .same } }

    private static func split(_ s: String) -> [String] {
        s.isEmpty ? [] : s.components(separatedBy: "\n")
    }

    private static func number(_ l: [(Kind, String)]) -> [Line] {
        l.enumerated().map { Line(id: $0.offset, kind: $0.element.0, text: $0.element.1) }
    }
}
