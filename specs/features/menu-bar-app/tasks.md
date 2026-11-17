# Tasks: Menu bar app

_Written after `plan.md` was approved on 2026-11-17. Strict prerequisite-first order. AI checks off items as work is completed._

Prerequisite: `git-sync`, `search-and-read`, and `updates` reach `implemented` with "Update at the start of every call". The other `requires` (`activity-and-undo`, `document-history`, `write-approval`) are `implemented`.

## Package and core

- [x] `apps/macos/Package.swift`: the targets `WaymarkBarCore`, `WaymarkBar`, and `WaymarkBarCoreTests`, macOS 14. `swift build` works.
- [x] `JSON.swift` and `MCPClient.swift`: start a process, `initialize`, `tools/list`, `tools/call`, answers by ID, server requests (`elicitation/create`) to a handler, process exit as an error for open calls.
- [x] `Binary.swift`: find `waymark` (setting, `/opt/homebrew/bin`, `/usr/local/bin`), the memory repository arguments, and the deny list. `BinaryTests.swift`.
- [x] `Models.swift`: decode the results of `status`, `search`, `show`, `history`, `activity`, `updates`, `undo`, the `settings` block, and the tool error (`code`, `message`, `next`).
- [x] `Session.swift`: start, the `tools/list` check and restart with a longer deny list, `status`, the `show` budget, restart after an exit with back-off, and one retry after `access_denied`.
- [x] `Fixture.swift` and the first `IntegrationTests.swift` cases: the deny list (`undo` is the only write tool), `status`, a missing binary.
- [x] `Diff.swift` and `DiffTests.swift`: line diff and changed properties.
- [x] `Links.swift` and `LinksTests.swift`: `[[id]]`, `[[id|label]]`, `[[id#part]]`, relative links with `../`, and `.md`.
- [x] `Watched.swift`, `Reviewed.swift`, `AppSettings.swift`, and `StoresTests.swift`.
- [x] `RecentWrites.swift`: merge `activity` pages, find the version before a write in `history`, load both versions, and build the diff. Integration tests: a new write is listed, the diff of an update, "Undone", only the person's own writes.
- [x] `Poller.swift` and `PollerTests.swift` (fake session): document `version`, the folder signal from `updates` and `activity`, full folder reads, no change means no event, the "Behind" state from a notice. Integration tests: a push by another person shows at the next poll for a document and for a folder, `updates` with `since` does not move the stored time of the agent tools.
- [x] Undo and approval in the core: preview, undo, refusal with `message` and `next`, the elicitation handler answers. Integration tests: preview changes nothing, undo pushes a revert under the person's name, a conflict refuses, `approval.required_for = ["undo"]` asks the handler and runs only after a yes, `access = "read-only"` hides Undo, the read budget restart.

## App

- [x] `App.swift`, `AppModel.swift`, `Strings.swift`, `Localizable.strings`, `Info.plist`: menu bar only, the icon states, the poll timer, the session from the settings.
- [x] `MenuView.swift` and `SettingsWindow.swift`: the menu, the watched list with dots, Refresh now, the setup help, the install help, Start at login.
- [x] `SearchWindow.swift`: search with More results, Watch for a document and a folder.
- [x] `MarkdownView.swift` and `DocumentWindow.swift`: render, banners, links, Back and Forward, "Updated just now by", deleted and moved.
- [x] `FolderWindow.swift`: the list, newest first, marks for new and changed.
- [x] `DiffView.swift`, `ActivityWindow.swift`, `UndoSheet.swift`, `ApprovalSheet.swift`: the recent writes window with the detail, review buttons, Load more, Undo, and the approval dialog.
- [x] Notifications: one per poll for watched items and one for new writes, off by default, a click opens the window.

## Build and release

- [x] `scripts/macos-app.sh`: build, bundle, sign ad hoc or with a Developer ID, and with `--dist` notarize and zip.
- [x] `Makefile`: `app` and `app-test`. `check` runs `app-test` on macOS.
- [x] `packaging/homebrew/waymark-bar.rb` and `scripts/release.sh --with-app`.
- [x] `README.md`: the app section.
- [x] Run the full suite (`make check`) and the app build.
