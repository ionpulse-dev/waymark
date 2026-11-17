import Foundation

/// A document or a folder that the person watches.
public struct WatchedItem: Codable, Equatable, Hashable, Sendable, Identifiable {
    public enum Kind: String, Codable, Sendable { case document, folder }

    public var kind: Kind
    /// The document ID, or the folder path ("" is the root).
    public var path: String
    /// The title of a document, for the menu.
    public var title: String

    public var id: String { "\(kind.rawValue):\(path)" }

    public init(kind: Kind, path: String, title: String = "") {
        self.kind = kind
        self.path = path
        self.title = title
    }

    public var label: String {
        if kind == .folder { return path.isEmpty ? "/" : path + "/" }
        return title.isEmpty ? path : title
    }
}

/// The watched list, stored with the settings. It survives a restart.
public final class WatchedStore: @unchecked Sendable {
    private let defaults: UserDefaults
    private let key = "watched"

    public init(defaults: UserDefaults = .standard) { self.defaults = defaults }

    public var items: [WatchedItem] {
        get {
            guard let d = defaults.data(forKey: key) else { return [] }
            return (try? JSONDecoder().decode([WatchedItem].self, from: d)) ?? []
        }
        set { defaults.set(try? JSONEncoder().encode(newValue), forKey: key) }
    }

    @discardableResult
    public func add(_ item: WatchedItem) -> Bool {
        var list = items
        if list.contains(where: { $0.id == item.id }) { return false }
        list.append(item)
        items = list
        return true
    }

    public func remove(_ item: WatchedItem) {
        items = items.filter { $0.id != item.id }
    }

    public func rename(_ item: WatchedItem, to path: String) {
        items = items.map { $0.id == item.id ? WatchedItem(kind: $0.kind, path: path, title: $0.title) : $0 }
    }
}
