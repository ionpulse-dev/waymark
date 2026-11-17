# Plan: Menu bar app

## Overview

A Swift app in this repository, in `apps/macos/`. It holds no document logic. It runs `waymark` as a child process, talks MCP over its standard input and output, and draws the windows. A poller calls the read tools at the interval and compares the results with the last poll.

## Components Affected

- **`waymark` (Rust):** no change. The app uses the existing tools and flags. It depends on the change "Update at the start of every call" of `git-sync` (planned on 2026-11-17).
- **`distribution`:** a Homebrew cask for the app in the same tap, signed and notarized. The cask depends on the `waymark` formula.
- **`README.md`:** a short section about the app.

## New Files

- `apps/macos/Package.swift` — the Swift package. The app target and a test target.
- `apps/macos/Sources/WaymarkBar/App.swift` — the app entry, `LSUIElement`, the menu bar item, and the login item (`SMAppService`).
- `apps/macos/Sources/WaymarkBar/MCPClient.swift` — starts `waymark`, sends JSON-RPC over stdio, matches answers to requests, and restarts the process after an exit or `access_denied`.
- `apps/macos/Sources/WaymarkBar/Binary.swift` — finds `waymark` and builds its arguments.
- `apps/macos/Sources/WaymarkBar/Poller.swift` — the timer, the calls per watched item, and the change detection.
- `apps/macos/Sources/WaymarkBar/Watched.swift` — the watched list and its storage in `UserDefaults`.
- `apps/macos/Sources/WaymarkBar/Views/MenuView.swift` — the menu and the icon states.
- `apps/macos/Sources/WaymarkBar/Views/SearchWindow.swift` — the search window.
- `apps/macos/Sources/WaymarkBar/Views/DocumentWindow.swift` — the document window, banners, links, Back and Forward.
- `apps/macos/Sources/WaymarkBar/Views/FolderWindow.swift` — the folder window.
- `apps/macos/Sources/WaymarkBar/Views/ActivityWindow.swift` — the recent writes window: the list, the detail view, and the review buttons.
- `apps/macos/Sources/WaymarkBar/Reviewed.swift` — the reviewed state of writes, stored in `UserDefaults`.
- `apps/macos/Sources/WaymarkBar/Diff.swift` — a line diff of two document versions, for display only.
- `apps/macos/Sources/WaymarkBar/Views/UndoSheet.swift` — the preview and confirm sheet of an undo.
- `apps/macos/Sources/WaymarkBar/Views/ApprovalSheet.swift` — the native sheet for an elicitation request from `waymark`.
- `apps/macos/Sources/WaymarkBar/Views/SettingsWindow.swift` — the settings window.
- `apps/macos/Sources/WaymarkBar/Markdown.swift` — renders the body, and turns `[[...]]` and relative links into document links.
- `apps/macos/Sources/WaymarkBar/Resources/en.lproj/Localizable.strings` — all window strings.
- `apps/macos/Tests/WaymarkBarTests/PollerTests.swift` — change detection with a fake MCP client.
- `apps/macos/Tests/WaymarkBarTests/MCPClientTests.swift` — a real `waymark` against a temporary memory repository.
- `scripts/macos-app.sh` — builds, signs, and notarizes the app bundle.

## Technical Approach

1. **UI framework.** SwiftUI with `MenuBarExtra` for the menu and `Window` scenes for the windows. AppKit only where SwiftUI lacks a feature.
2. **MCP client.** A small client of our own: `initialize`, `tools/list`, `tools/call`, line-based JSON-RPC over the pipes of `Process`. It announces the `elicitation` capability and answers `elicitation/create` requests with a native dialog (see step 9). The app uses only a few tools, so a full MCP SDK is not needed. See `decisions.md`.
3. **Process.** One `waymark` process for the app. Arguments: the memory repository arguments from the settings, `--max-documents-opened 500`, and `--tools-denied` with every write tool except `undo`. After `initialize`, the client reads `tools/list`. If a tool with `readOnlyHint: false` other than `undo` is listed, it adds that tool to the list and starts the process again. So a newer `waymark` with a new write tool never gives the app more than `undo`. If the process exits, the client starts it again with a back-off (1, 2, 4, up to 60 seconds).
4. **Poller.** A timer at the interval. One poll runs the calls one after another, not in parallel, so that the lock is not contested. "Refresh now" runs a poll at once. A poll does not start while one runs.
5. **Change detection.** Per document: the last `version`. Per folder: the map from ID to `updated`, from `search` with paging. A difference marks the item and updates its window. For a folder, `updates` with `since` gives the names of the people who changed it.
6. **Recent writes.** Each poll calls `activity` with `limit: 50` and pages with `before` until it reaches the newest commit it already knows. The first start reads one page only. The app keeps the commits in memory and stores only the reviewed set (commit hashes, at most the last 1000).
7. **Diff of one write.** For each ID of the write: `history` of the document, then the entry just after the commit of the write (the version before). `show` with `version` for both. A line diff of the bodies plus a list of changed properties. The diff is display logic only. It does not parse or check documents.
8. **Undo.** The detail view of a write has an Undo button when the write is not undone and writes are allowed. A click calls `undo` with `preview: true` and shows the `changes` in a sheet. **Undo** in the sheet calls `undo` without `preview`. After the result, the poller reads `activity` again, so the list shows "Undone" and the new undo commit. The app marks both as reviewed.
9. **Approval in the app.** An `elicitation/create` request from `waymark` (see `write-approval`) opens a native sheet with the message of the request and **Approve** and **Decline**. The answer goes back as the elicitation result. No answer before the time-out of `waymark` counts as decline.
10. **Writes allowed?** From `status` at start: the effective `access`, `paused`, and `tools_denied`. The window hides or disables Undo as `requirements.md` says, and checks again after an `access_denied` or `paused` error.
11. **State of the icon.** From the last poll: an error, a notice that the clone may be behind, or normal.
12. **Storage.** Settings and the watched list in `UserDefaults`. No document content is stored on disk. The windows keep the last result in memory.
13. **Rendering.** `AttributedString(markdown:)` for the body, plus a step that turns wiki links into `waymark-doc:` URLs that the window handles.

## Dependencies on Other Specs

- `git-sync` — "Update at the start of every call". Without it, a poll can be up to 3 minutes behind.
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
- **Polling cost** grows with the watched items. Mitigation: calls one after another, and the minimum interval of 10 seconds.

## Open Questions

See `requirements.md`.
