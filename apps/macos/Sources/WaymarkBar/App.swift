import AppKit
import SwiftUI
import UserNotifications
import WaymarkBarCore

/// The menu bar app of waymark (see `specs/features/menu-bar-app`).
@main
struct WaymarkBarApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var delegate
    @State private var model = AppModel.shared

    var body: some Scene {
        MenuBarExtra {
            MenuView()
                .environment(model)
        } label: {
            MenuLabel()
                .environment(model)
        }
        .menuBarExtraStyle(.menu)

        Window(L("Search"), id: "search") {
            SearchWindow().environment(model)
        }
        .defaultSize(width: 520, height: 480)

        WindowGroup(L("Document"), id: "document", for: String.self) { $id in
            DocumentWindow(startID: id ?? "").environment(model)
        }
        .defaultSize(width: 640, height: 720)

        WindowGroup(L("Folder"), id: "folder", for: String.self) { $path in
            FolderWindow(path: path ?? "").environment(model)
        }
        .defaultSize(width: 560, height: 600)

        Window(L("Recent writes"), id: "activity") {
            ActivityWindow().environment(model)
        }
        .defaultSize(width: 980, height: 640)

        Settings {
            SettingsWindow().environment(model)
        }
    }
}

final class AppDelegate: NSObject, NSApplicationDelegate, UNUserNotificationCenterDelegate {
    func applicationDidFinishLaunching(_ notification: Notification) {
        // Menu bar only: no Dock icon (also set by LSUIElement in Info.plist).
        NSApp.setActivationPolicy(.accessory)
        if Bundle.main.bundleIdentifier != nil {
            UNUserNotificationCenter.current().delegate = self
        }
        Task { @MainActor in AppModel.shared.start() }
    }

    func userNotificationCenter(
        _ center: UNUserNotificationCenter, didReceive response: UNNotificationResponse,
        withCompletionHandler completionHandler: @escaping () -> Void
    ) {
        let target = response.notification.request.content.userInfo["target"] as? String ?? ""
        Task { @MainActor in AppModel.shared.openTarget(target) }
        completionHandler()
    }

    func userNotificationCenter(
        _ center: UNUserNotificationCenter, willPresent notification: UNNotification,
        withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void
    ) {
        completionHandler([.banner, .sound])
    }
}

/// macOS notifications (off by default, see "Notifications" in `requirements.md`).
enum Notifier {
    static var available: Bool { Bundle.main.bundleIdentifier != nil }

    static func requestPermission() {
        guard available else { return }
        UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound]) { _, _ in }
    }

    static func post(title: String, body: String, target: String) {
        guard available else { return }
        let c = UNMutableNotificationContent()
        c.title = title
        c.body = body
        c.userInfo = ["target": target]
        UNUserNotificationCenter.current().add(
            UNNotificationRequest(identifier: UUID().uuidString, content: c, trigger: nil))
    }
}
