import Foundation

/// The settings of the app (see "Settings window" in `requirements.md`).
public struct AppSettings: Codable, Equatable, Sendable {
    public static let minInterval: TimeInterval = 10
    public static let maxInterval: TimeInterval = 3600

    public var repository = RepositorySetting()
    /// Seconds between two polls, 10 to 3600.
    public var pollInterval: TimeInterval = 60
    public var notifications = false
    public var startAtLogin = false
    /// A path to `waymark`. Empty: the app finds it.
    public var binaryPath = ""

    public init() {}

    public var clampedInterval: TimeInterval {
        min(Self.maxInterval, max(Self.minInterval, pollInterval))
    }
}

public final class SettingsStore: @unchecked Sendable {
    private let defaults: UserDefaults
    private let key = "settings"

    public init(defaults: UserDefaults = .standard) { self.defaults = defaults }

    public var settings: AppSettings {
        get {
            guard let d = defaults.data(forKey: key),
                let s = try? JSONDecoder().decode(AppSettings.self, from: d)
            else { return AppSettings() }
            return s
        }
        set { defaults.set(try? JSONEncoder().encode(newValue), forKey: key) }
    }
}
