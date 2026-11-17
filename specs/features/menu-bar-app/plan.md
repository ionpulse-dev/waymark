# Plan: Menu bar app

## Overview

A Swift app in this repository, in `apps/macos/`. It holds no document logic. It runs `waymark` as a child process, talks MCP over its standard input and output, and draws the windows. A poller calls the read tools at the interval and compares the results with the last poll.

## Components Affected

- **`waymark` (Rust):** no change. The app uses the existing tools and flags. It depends on the change "Update at the start of every call" of `git-sync` (implemented on 2026-11-17).
- **`distribution`:** `scripts/release.sh` gets an optional `--with-app` step: it builds, signs, and notarizes the app, attaches `WaymarkBar.zip` to the release, and updates a Homebrew cask in the same tap. Without `--with-app` the release is as before. `--with-app` fails if the Developer ID identity or the `notarytool` profile is missing.
- **`Makefile`:** `app-test` (runs `swift test` in `apps/macos`) and `app` (builds the bundle). `check` runs `app-test` on macOS.
- **`README.md`:** a short section about the app.

## New Files

- `apps/macos/.gitignore` — the build folders of SwiftPM and of the bundle script.
- `apps/macos/Package.swift` — the Swift package: the library `WaymarkBarCore`, the executable `WaymarkBar`, and the test target.
- `apps/macos/Sources/WaymarkBarCore/JSON.swift` — a small JSON value type for the MCP messages.
- `apps/macos/Sources/WaymarkBarCore/MCPClient.swift` — line-based JSON-RPC over the pipes of a `Process`: requests, answers, and server requests (`elicitation/create`).
- `apps/macos/Sources/WaymarkBarCore/Binary.swift` — finds `waymark` and builds its arguments, with the deny list.
- `apps/macos/Sources/WaymarkBarCore/Session.swift` — one `waymark` session: start, the check of `tools/list`, the `show` budget, restart with back-off, and the typed tool calls.
- `apps/macos/Sources/WaymarkBarCore/Models.swift` — the results of `status`, `search`, `show`, `history`, `activity`, `updates`, and `undo`, and the tool error.
- `apps/macos/Sources/WaymarkBarCore/Poller.swift` — one poll: `activity`, `updates` per folder, `show` per document, full folder reads, and the change detection.
- `apps/macos/Sources/WaymarkBarCore/Watched.swift` — the watched list and its storage.
- `apps/macos/Sources/WaymarkBarCore/Reviewed.swift` — the reviewed set of writes (at most 1000 commit hashes).
- `apps/macos/Sources/WaymarkBarCore/RecentWrites.swift` — merges `activity` pages, and loads the change of one write (versions before and after).
- `apps/macos/Sources/WaymarkBarCore/Diff.swift` — a line diff of two texts and the changed properties, for display only.
- `apps/macos/Sources/WaymarkBarCore/Links.swift` — turns `[[...]]` and relative Markdown links into document IDs and `waymark-doc:` URLs.
- `apps/macos/Sources/WaymarkBarCore/AppSettings.swift` — the settings and their storage in `UserDefaults`.
- `apps/macos/Sources/WaymarkBar/App.swift` — the app entry, `MenuBarExtra`, the window scenes, and the login item (`SMAppService`).
- `apps/macos/Sources/WaymarkBar/AppModel.swift` — the observable state of the app: the session, the poll timer, the icon state, the notifications.
- `apps/macos/Sources/WaymarkBar/Strings.swift` — the access to the strings of `Localizable.strings`.
- `apps/macos/Sources/WaymarkBar/Views/MenuView.swift` — the menu and the icon states.
- `apps/macos/Sources/WaymarkBar/Views/SearchWindow.swift` — the search window.
- `apps/macos/Sources/WaymarkBar/Views/DocumentWindow.swift` — the document window, banners, links, Back and Forward.
- `apps/macos/Sources/WaymarkBar/Views/FolderWindow.swift` — the folder window.
- `apps/macos/Sources/WaymarkBar/Views/ActivityWindow.swift` — the recent writes window: the list, the detail view, the review buttons, and Undo.
- `apps/macos/Sources/WaymarkBar/Views/DiffView.swift` — shows a diff.
- `apps/macos/Sources/WaymarkBar/Views/UndoSheet.swift` — the preview and confirm sheet of an undo.
- `apps/macos/Sources/WaymarkBar/Views/ApprovalSheet.swift` — the native dialog for an elicitation request from `waymark`.
- `apps/macos/Sources/WaymarkBar/Views/SettingsWindow.swift` — the settings window and the setup help.
- `apps/macos/Sources/WaymarkBar/Views/MarkdownView.swift` — renders the body with `AttributedString(markdown:)` and the document links.
- `apps/macos/Sources/WaymarkBar/Resources/en.lproj/Localizable.strings` — all window strings.
- `apps/macos/Info.plist` — the bundle facts: `LSUIElement`, the minimum macOS version, the bundle ID.
- `apps/macos/Tests/WaymarkBarCoreTests/DiffTests.swift` — the line diff.
- `apps/macos/Tests/WaymarkBarCoreTests/LinksTests.swift` — the link rules.
- `apps/macos/Tests/WaymarkBarCoreTests/StoresTests.swift` — the reviewed set, the watched list, and the settings survive a restart.
- `apps/macos/Tests/WaymarkBarCoreTests/PollerTests.swift` — change detection with a fake session.
- `apps/macos/Tests/WaymarkBarCoreTests/BinaryTests.swift` — finding `waymark` and the arguments.
- `apps/macos/Tests/WaymarkBarCoreTests/MCPClientTests.swift` — the client against a process that stops at once, and a missing binary.
- `apps/macos/Tests/WaymarkBarCoreTests/Fixture.swift` — a temporary memory repository with a bare remote and two clones, for the integration tests.
- `apps/macos/Tests/WaymarkBarCoreTests/IntegrationTests.swift` — a real `waymark` against the fixture: the deny list, polls, recent writes, the diff, Undo, approval, and the read budget.
- `scripts/macos-app.sh` — builds the app bundle, signs it, and with `--dist` notarizes it and makes `WaymarkBar.zip`.
- `packaging/homebrew/waymark-bar.rb` — the cask template for the tap.

## Technical Approach

1. **UI framework.** SwiftUI with `MenuBarExtra` (window style) for the menu and `Window`/`WindowGroup` scenes for the windows. AppKit only where SwiftUI lacks a feature (bringing a window to the front, the Dock icon policy).
2. **MCP client.** A small client of our own: `initialize`, `notifications/initialized`, `tools/list`, `tools/call`, line-based JSON-RPC over the pipes of `Process`. It announces the `elicitation` capability (form mode) and answers `elicitation/create` with a handler that the app sets (see step 9). Calls are `async` and matched by ID. See `decisions.md`.
3. **Session.** One `waymark` process at a time. Arguments: the memory repository arguments from the settings and `--tools-denied` with every write tool except `undo`. No number limits (see `decisions.md`). After `initialize`, the session reads `tools/list`. If a tool with `readOnlyHint: false` other than `undo` is listed, it adds that tool to the deny list and starts again. Then `status` gives the limits. The session counts the distinct documents that `show` opened and starts a new process before the count reaches `max_documents_opened`. A replaced process stops after its last open call. An `access_denied` from `show` starts a new process and tries once. A process that exits is started again on the next call, with a back-off (1, 2, 4, up to 60 seconds).
4. **Poller.** A timer at the interval. One poll runs the calls one after another, not in parallel. "Refresh now" runs a poll at once, with full folder reads. A poll does not start while one runs.
5. **Change detection.** Per document: the last `version`, from `show`. Per folder: `updates` with `since` (the `until` of the last `updates` call of that folder) and `folders`, all pages, plus the `activity` entries of the poll with that folder or a subfolder. A hit, an opened window, or "Refresh now" reads the folder in full with paged `search` (`level: 2` for the description, `status: "any"`), and compares the map from ID to `updated` and title with the last one. New and changed IDs are marked.
6. **Recent writes.** Each poll calls `activity` with `limit: 50` and pages with `before` until it reaches the newest commit it already knows (at most 5 pages per poll). The first start reads one page. The app keeps the entries in memory and stores only the reviewed set (commit hashes, at most the last 1000). On the first start, all listed writes count as reviewed, so the badge starts at 0.
7. **Change of one write.** For each ID of the write: `history` of the document, paged until the entry of the write is found. The entry after it is the version before. `show` with `version` for both. A line diff of the bodies plus a list of changed properties (title, description, status, tags, disputed). For an `add`, only the version of the write. For a `delete` or `move`, no `show`.
8. **Undo.** The detail view of a write has an Undo button when the write is not undone and writes are allowed. A click calls `undo` with `preview: true` and shows the `changes` in a sheet. **Undo** in the sheet calls `undo` without `preview`. Then the poller reads `activity` again, so the list shows "Undone" and the new undo commit. The app marks both as reviewed.
9. **Approval in the app.** An `elicitation/create` request from `waymark` opens a native dialog with the message of the request and **Approve** and **Decline**. **Approve** answers `accept` with `approve: true`. **Decline** answers `decline`. Closing it answers `cancel`. `waymark` counts no answer in 2 minutes as a no.
10. **Writes allowed?** From the `settings` that every result carries: `access`, `paused`, and `tools_denied`. Undo is hidden for `read-only` or a denied `undo`, and disabled while `paused`.
11. **State of the icon.** From the last poll: an error, a notice that the clone may be behind, or normal.
12. **Storage.** Settings, the watched list, and the reviewed set in `UserDefaults`. No document content is stored on disk. The windows keep the last result in memory.
13. **Rendering.** `AttributedString(markdown:)` per paragraph, after a step that turns wiki links and relative links into `waymark-doc:` URLs that the window handles.
14. **Bundle and release.** `scripts/macos-app.sh` (see `decisions.md`). The cask installs `WaymarkBar.app` and depends on the `waymark` formula.

## Dependencies on Other Specs

- `git-sync` — "Update at the start of every call". Without it, a poll can be up to 3 minutes behind.
- `trust-model` — the limits. The app works within them.
- `search-and-read` — `search`, `show`, `status`, and the read budget.
- `updates` — `updates` with `since` and `folders`.
- `activity-and-undo` — `activity` with `limit` and `before`.
- `document-history` — `history`, and `show` with `version`.
- `write-approval` — the elicitation request when a rule covers `undo`.
- `distribution` — the tap and the release steps.

## Risks

- **Signing and notarization** need an Apple Developer ID. That is an outward step for the product owner.
- **The maintainer's build** must work with the Swift version of the Xcode that the maintainer has. CI for the app comes later.
- **Markdown rendering** with `AttributedString` is basic (no tables). Accepted for the first version. A better renderer can come later.
- **A wrong deny list** would let the app call a write tool. Mitigation: the check of `tools/list` at every start, and a test that fails when `waymark` has a write tool that the app's list lacks.
- **No Developer ID on the maintainer's Mac** (on 2026-11-17). The app builds and runs signed ad hoc on that Mac. A notarized build for other Macs waits for the identity.
- **Polling cost** grows with the watched items. Mitigation: calls one after another, the cheap folder signal, and the minimum interval of 10 seconds.

## Open Questions

See `requirements.md`.
