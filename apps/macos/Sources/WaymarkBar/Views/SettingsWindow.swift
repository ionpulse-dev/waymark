import AppKit
import SwiftUI
import WaymarkBarCore

/// The settings window and the setup help (see "Settings window" in `requirements.md`).
struct SettingsWindow: View {
    @Environment(AppModel.self) private var model
    @State private var location = ""
    @State private var cloneFolder = ""
    @State private var repositoryID = ""
    @State private var binaryPath = ""
    @State private var interval = 60.0

    var body: some View {
        @Bindable var model = model
        Form {
            Section(L("Memory repository")) {
                if model.settings.repository.isEmpty { setupHelp }
                HStack {
                    TextField(L("Git URL or folder"), text: $location, prompt: Text("git@github.com:you/memory.git"))
                    Button(L("Choose Folder…")) { chooseFolder() }
                }
                TextField(L("Clone folder (optional, for a URL)"), text: $cloneFolder, prompt: Text("~/.waymark/memory"))
                TextField(L("Memory repository ID (optional)"), text: $repositoryID)
                HStack {
                    Spacer()
                    Button(L("Apply")) { applyRepository() }
                        .keyboardShortcut(.defaultAction)
                        .disabled(!changed)
                }
            }
            Section(L("Polling")) {
                HStack {
                    Slider(value: $interval, in: AppSettings.minInterval...AppSettings.maxInterval, step: 10) {
                        Text(L("Poll interval"))
                    } onEditingChanged: { editing in
                        if !editing { model.settings.pollInterval = interval }
                    }
                    Text(intervalText).monospacedDigit().frame(width: 80, alignment: .trailing)
                }
                Text(L("Every poll fetches from the Git host. A short interval with many watched items means many fetches."))
                    .font(.caption).foregroundStyle(.secondary)
            }
            Section(L("Mac")) {
                Toggle(L("Notifications"), isOn: $model.settings.notifications)
                Toggle(L("Start at login"), isOn: $model.settings.startAtLogin)
                if model.settings.startAtLogin {
                    Text(model.loginItemStatus).font(.caption).foregroundStyle(.secondary)
                }
            }
            Section(L("waymark binary")) {
                TextField(L("Path (empty: found by the app)"), text: $binaryPath, prompt: Text("/opt/homebrew/bin/waymark"))
                    .onSubmit { model.settings.binaryPath = binaryPath }
                Text(binaryLine).font(.caption).foregroundStyle(.secondary)
                if model.setup == .notInstalled {
                    Text(L("waymark is not installed. Install it with Homebrew, then choose Retry."))
                    Text("brew install ionpulse-dev/tap/waymark").font(.system(.body, design: .monospaced)).textSelection(.enabled)
                    Button(L("Retry")) {
                        model.settings.binaryPath = binaryPath
                        model.retry()
                    }
                }
            }
        }
        .formStyle(.grouped)
        .frame(width: 560)
        .padding(.vertical, 8)
        .onAppear(perform: load)
    }

    private var setupHelp: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(L("Set up the app")).font(.headline)
            Text(L("Enter the same memory repository as in the MCP entry of your agent tool: the Git URL of --repo, or the folder of --path. The app starts its own waymark with it and only reads. Its only write is Undo of your own writes."))
                .fixedSize(horizontal: false, vertical: true)
        }
        .padding(.vertical, 4)
    }

    private var changed: Bool {
        let r = model.settings.repository
        return r.location != location || r.cloneFolder != cloneFolder || r.memoryRepositoryID != repositoryID
    }

    private var intervalText: String {
        let s = Int(interval)
        return s < 60 ? L("%d s", s) : (s % 60 == 0 ? L("%d min", s / 60) : L("%d min %d s", s / 60, s % 60))
    }

    private var binaryLine: String {
        let setting = binaryPath.isEmpty ? nil : binaryPath
        if let u = Binary.find(setting: setting) {
            let v = model.serverVersion.map { " (\($0))" } ?? ""
            return L("In use: %@", u.path) + v
        }
        return L("Not found. The app looks in /opt/homebrew/bin, then /usr/local/bin.")
    }

    private func load() {
        let s = model.settings
        location = s.repository.location
        cloneFolder = s.repository.cloneFolder
        repositoryID = s.repository.memoryRepositoryID
        binaryPath = s.binaryPath
        interval = s.clampedInterval
    }

    private func applyRepository() {
        model.settings.repository = RepositorySetting(
            location: location.trimmingCharacters(in: .whitespaces),
            cloneFolder: cloneFolder.trimmingCharacters(in: .whitespaces),
            memoryRepositoryID: repositoryID.trimmingCharacters(in: .whitespaces))
        model.settings.binaryPath = binaryPath
    }

    private func chooseFolder() {
        let panel = NSOpenPanel()
        panel.canChooseDirectories = true
        panel.canChooseFiles = false
        panel.allowsMultipleSelection = false
        panel.prompt = L("Choose")
        NSApp.activate(ignoringOtherApps: true)
        if panel.runModal() == .OK, let u = panel.url { location = u.path }
    }
}
