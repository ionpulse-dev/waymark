import Foundation

/// The writes that the person marked as reviewed. Stored on this Mac only (see `decisions.md`).
public final class ReviewedStore: @unchecked Sendable {
    public static let maxCount = 1000
    private let defaults: UserDefaults
    private let key = "reviewed"
    private let startedKey = "reviewedStarted"
    private let lock = NSLock()

    public init(defaults: UserDefaults = .standard) { self.defaults = defaults }

    /// The hashes, oldest first.
    public var commits: [String] { lock.withLock { defaults.stringArray(forKey: key) ?? [] } }

    public func isReviewed(_ commit: String) -> Bool { commits.contains(commit) }

    public func mark(_ list: [String]) {
        lock.withLock {
            var c = defaults.stringArray(forKey: key) ?? []
            for h in list where !c.contains(h) { c.append(h) }
            if c.count > Self.maxCount { c.removeFirst(c.count - Self.maxCount) }
            defaults.set(c, forKey: key)
        }
    }

    public func unmark(_ commit: String) {
        lock.withLock {
            defaults.set((defaults.stringArray(forKey: key) ?? []).filter { $0 != commit }, forKey: key)
        }
    }

    /// Has the app read the writes before? On the first start all listed writes count as reviewed.
    public var hasStarted: Bool {
        get { defaults.bool(forKey: startedKey) }
        set { defaults.set(newValue, forKey: startedKey) }
    }

    /// Forget all, for a new memory repository.
    public func reset() {
        lock.withLock {
            defaults.removeObject(forKey: key)
            defaults.removeObject(forKey: startedKey)
        }
    }
}
