import Foundation

/// A JSON value, for the MCP messages.
public enum JSON: Equatable, Sendable {
    case null
    case bool(Bool)
    case number(Double)
    case string(String)
    case array([JSON])
    case object([String: JSON])

    public subscript(key: String) -> JSON {
        if case .object(let o) = self { return o[key] ?? .null }
        return .null
    }

    public subscript(index: Int) -> JSON {
        if case .array(let a) = self, index >= 0, index < a.count { return a[index] }
        return .null
    }

    public var string: String? { if case .string(let s) = self { return s }; return nil }
    public var bool: Bool? { if case .bool(let b) = self { return b }; return nil }
    public var double: Double? { if case .number(let n) = self { return n }; return nil }
    public var int: Int? { double.map { Int($0) } }
    public var array: [JSON] { if case .array(let a) = self { return a }; return [] }
    public var object: [String: JSON]? { if case .object(let o) = self { return o }; return nil }
    public var isNull: Bool { self == .null }
    public var strings: [String] { array.compactMap(\.string) }

    /// Parse JSON text.
    public static func parse(_ data: Data) throws -> JSON {
        try from(JSONSerialization.jsonObject(with: data, options: [.fragmentsAllowed]))
    }

    public static func parse(_ text: String) throws -> JSON {
        try parse(Data(text.utf8))
    }

    static func from(_ any: Any) -> JSON {
        switch any {
        case is NSNull: return .null
        case let n as NSNumber:
            // NSNumber holds both booleans and numbers.
            if CFGetTypeID(n) == CFBooleanGetTypeID() { return .bool(n.boolValue) }
            return .number(n.doubleValue)
        case let s as String: return .string(s)
        case let a as [Any]: return .array(a.map(from))
        case let o as [String: Any]: return .object(o.mapValues(from))
        default: return .null
        }
    }

    var foundation: Any {
        switch self {
        case .null: return NSNull()
        case .bool(let b): return b
        case .number(let n):
            if n.rounded() == n, abs(n) < 1e15 { return Int(n) }
            return n
        case .string(let s): return s
        case .array(let a): return a.map(\.foundation)
        case .object(let o): return o.mapValues(\.foundation)
        }
    }

    /// One line of JSON text, with no line breaks (the stdio transport of MCP).
    public func serialized() -> Data {
        (try? JSONSerialization.data(
            withJSONObject: foundation, options: [.fragmentsAllowed, .sortedKeys, .withoutEscapingSlashes]))
            ?? Data("null".utf8)
    }

    public var text: String { String(decoding: serialized(), as: UTF8.self) }
}

extension JSON: ExpressibleByStringLiteral, ExpressibleByBooleanLiteral, ExpressibleByIntegerLiteral,
    ExpressibleByArrayLiteral, ExpressibleByDictionaryLiteral, ExpressibleByNilLiteral
{
    public init(stringLiteral value: String) { self = .string(value) }
    public init(booleanLiteral value: Bool) { self = .bool(value) }
    public init(integerLiteral value: Int) { self = .number(Double(value)) }
    public init(arrayLiteral elements: JSON...) { self = .array(elements) }
    public init(dictionaryLiteral elements: (String, JSON)...) {
        self = .object(Dictionary(elements, uniquingKeysWith: { _, b in b }))
    }
    public init(nilLiteral: ()) { self = .null }
}

extension JSON {
    public init(_ strings: [String]) { self = .array(strings.map { .string($0) }) }
}
