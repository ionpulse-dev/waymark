# Decisions: Menu bar app

## A new spec, not a restore of `mac-app` — 2026-11-17

**Decision:** The menu bar app gets a new feature spec. The superseded specs `mac-app`, `app-link`, and `approval-dialog` are not restored. On the same day the product owner removed them (see "Superseded specs can be removed; the old Mac app specs are removed" in [project decisions](../../../docs/decisions.md)). Their last version is in commit `1e588d9`.

**Alternatives considered:** Set `mac-app` back to `draft` (rejected: it never had a plan, tasks, or tests, and it is built on the CLI, the app link socket, exit codes, and settings that only the app can change. None of these exist now. A restore would also bring back the native parts of about 15 feature specs that the MCP-only rewrite removed.)

**Reason:** The shell decisions of `mac-app` still hold. The architecture does not. A new spec takes the first and leaves the second.

**Kept from `mac-app`:** menu bar only with no Dock icon, native Swift, a direct download that is signed and notarized and not in the Mac App Store, start at login, icon states, all window strings in resource files, no document or Git logic in the app.

## The app is an MCP client of `waymark` — 2026-11-17

**Decision:** The app starts its own `waymark` process and talks MCP over stdio. It uses only existing tools.

**Alternatives considered:**
- Read the files of the clone and run `git` in the app (rejected: it copies document and Git logic into a second language, against "MCP is the only interface").
- A new socket between `waymark` and the app (rejected: that is the old app link, with its own protocol to keep).

**Reason:** MCP is the only interface. The app gets the same rules (limits, folders, read budget) as every other client for free.

## Read only in the first version — 2026-11-17

**Decision:** The app starts `waymark` with `--access read-only` and has no write action.

**Reason:** A write needs approval and an author, and the approval channel of the old design (the app link) is gone. Reading covers the user story. Writing is an open question.

**Superseded by:** "Undo is the only write — 2026-11-17".

## Polling, not subscriptions — 2026-11-17

**Decision:** The app polls at an interval it sets itself (10 seconds to 1 hour, default 60). `waymark` gets no timer, no MCP resources, and no subscriptions.

**Alternatives considered:** A background update setting in `waymark` and MCP resource subscriptions (the draft `watch-changes`, dropped on 2026-11-17 before it was committed). Rejected: with "Update at the start of every call", a poll is always fresh, and polling needs no change to `waymark`.

**Reason:** No new code in `waymark`, no background job in local mode, and the interval belongs to the client that needs it.

## A small MCP client of our own — 2026-11-17

**Decision:** The app has its own small JSON-RPC client over stdio, not an MCP SDK.

**Reason:** The app calls a few read tools (`search`, `show`, `updates`, `activity`, `history`, `status`). A small client has no dependency to update and is easy to test against a real `waymark`. Decided as a draft proposal. Open for review.

## The app raises its own read budget — 2026-11-17

**Superseded by:** "The app works within the limits, and starts a new session for the read budget — 2026-11-17".

**Decision:** The app passes `--max-documents-opened 500`. After `access_denied` it starts a new `waymark` process and tries once more.

**Reason:** The default of 20 is for an agent session, which is short. The app runs all day and opens many documents. The flag is a layer of the person. A stricter layer, such as the shared settings of the team, still wins. The restart is only a fallback for that case.

## Recent writes for a fast review — 2026-11-17

**Decision:** The app has a recent writes window. It lists the writes of the person from `activity`, shows each change as a diff, and lets the person mark writes as reviewed. The reviewed state stays on the Mac.

**Alternatives considered:**
- Show the writes in the folder windows only (rejected: the person must watch every folder, and writes outside them are missed).
- A new tool in `waymark` that returns a diff (rejected for now: `history` and `show` with `version` give both versions, and a diff for display is UI work).
- Store the reviewed state in the memory repository (rejected: it is personal and would make commits for nothing).

**Reason:** The product owner asked for a fast review for people who let their agents write on their own. `activity` already gives exactly the writes under the person's name, from every agent tool. The app adds the overview and the diff.

## Undo is the only write — 2026-11-17

**Supersedes:** "Read only in the first version — 2026-11-17".

**Decision:** The recent writes window has an Undo button. The app's `waymark` process has every write tool switched off with `--tools-denied`, except `undo`. An undo always shows its preview first and runs only after the person confirms. If an approval rule covers `undo`, the app answers the MCP elicitation request itself, as a native dialog.

**Alternatives considered:**
- Stay read only, and tell the person to ask an agent to undo (rejected: a review that cannot act is slow, and the point is a fast review).
- Start a second `waymark` process for writes only (rejected: two processes for one app, with no gain over the deny list).
- `--access read-only` and a special exception for `undo` in `waymark` (rejected: it needs a change to the limits of `trust-model`).

**Reason:** The product owner asked for Undo on 2026-11-17. Undo is the safety net of `activity-and-undo`, and it is safe: it only reverts the person's own writes, it is itself a commit that can be undone, and it never overwrites newer changes. The old approval problem does not apply: a click in the app is the person, not an agent, and the elicitation of `write-approval` already reaches the app as the MCP client.

## Product owner approval and the answers to the open questions — 2026-11-17

**Decision:** The product owner approved the spec and asked to build it end to end, run all automated tests, and prepare a release that stops before anything is published. The open questions are answered as follows (decided autonomously; the product owner can change any of them):
- **Personal server over HTTPS:** not in the first version (see `todo.md`).
- **Cheaper folder signal:** yes, but with no change to `waymark`. See "Folders are polled through `updates` and `activity`".
- **Minimum macOS version:** macOS 14. `MenuBarExtra` with the window style, `SMAppService`, and the SwiftUI `Window` scene need macOS 13 or later. macOS 14 adds `@Observable` and is two versions behind the current one.
- **More writes:** not in the first version. Undo stays the only write.
- **Reviewed state across Macs:** no. It stays on one Mac.
- **Our own small MCP client** (the draft decision above) is confirmed.

## The app works within the limits, and starts a new session for the read budget — 2026-11-17

**Supersedes:** "The app raises its own read budget — 2026-11-17".

**Decision:** The app passes no number limit to `waymark`. It reads `max_documents_opened` and `max_results` from `status`. It counts its `show` calls and starts a new `waymark` process before the count reaches the budget. An `access_denied` from `show` still starts a new process and tries once more.

**Alternatives considered:** `--max-documents-opened 500` as drafted (rejected: a test showed that a layer can only tighten a number limit, as `trust-model` says. The built-in default of 20 is the top. The flag had no effect). A change to `trust-model` that lets a person raise the limit (rejected: it loosens a rule that exists for agents, for one client).

**Reason:** The read budget protects the context window of an agent session. The app is not an agent, and each process is a new session by the rules of `search-and-read`. A restart costs one `initialize` and one `tools/list`. A team that wants to stop it can set `access` or `folders` in a layer that the app cannot change.

## Folders are polled through `updates` and `activity` — 2026-11-17

**Decision:** A poll of a watched folder calls `updates` with `since` and `folders`. The same poll's `activity` gives the person's own writes. Only when one of them names the folder, or when its window opens, or after "Refresh now", the app reads the full list with paged `search`.

**Alternatives considered:**
- Paged `search` at every poll, as drafted (rejected: `search` returns at most `max_results` per call, 5 by default. A folder of 200 documents needs 40 calls, and every call fetches).
- A `version` field in the `search` entries (rejected for now: a change to `search-and-read`, and it does not fix the paging).

**Reason:** One `updates` call per folder per poll, plus one `activity` call for all folders. Both name exactly what changed and who changed it. A manual edit that is not committed shows at the next full read.

## A core library and an app target — 2026-11-17

**Decision:** The Swift package has two targets: `WaymarkBarCore` (the MCP client, the process, the poller, the models, the diff, the link rules, the stores, with no SwiftUI) and `WaymarkBar` (the SwiftUI app). The tests use the core only.

**Reason:** The core can be tested with `swift test` from the command line, against a real `waymark`, with no UI and no signing. The windows stay thin.

## The app bundle is built by a script, not an Xcode project — 2026-11-17

**Decision:** `scripts/macos-app.sh` builds the package with `swift build -c release`, puts the binary into `WaymarkBar.app` with an `Info.plist` (`LSUIElement`), and signs it. With a Developer ID identity and a `notarytool` profile it also signs for distribution and notarizes. Without them it signs ad hoc for this Mac and says so.

**Alternatives considered:** An Xcode project (rejected: a large generated file in the repository, and the build needs the Xcode UI or `xcodebuild` settings that are hard to review).

**Reason:** One script that the release can run. The maintainer's Mac had no Developer ID identity on 2026-11-17, so the ad hoc path is the one that runs now.

## Notes from the core tests — 2026-11-17

- A result carries the limits that `waymark` read at the start of the call, before its pull. A change of the shared settings shows from the next call. The app uses the limits of the last result, so the Undo button follows one poll later. An undo in between still gets the refusal of `waymark`.
- The elicitation capability is announced as `{"form": {}}`. `waymark` accepts it (test A16).
- Two `waymark` bugs that the tests found are in [git-sync/todo.md](../../foundation/git-sync/todo.md). Neither blocks the app.

## "Behind" stays until `status` reports a good fetch — 2026-11-17

**Decision:** The poller keeps the state "Behind" from the first notice that the clone may be behind until `status` reports a `last_pull` after that time. While it is "Behind", each poll ends with one `status` call.

**Alternatives considered:** Read the state from the notices of each poll (rejected: `waymark` removes repeated notices in a session, so the second poll without network had no notice and showed "normal". Found in a real run of the app).

**Reason:** Decided autonomously. The icon must show "Behind" for as long as the remote cannot be reached.

## A debug file for support and tests — 2026-11-17

**Decision:** When the default `debugLogFile` holds a path (`defaults write dev.waymark.bar debugLogFile <path>`), the app appends one line per poll: the state, the watched items, the counts, and the notices. Off by default. No document text is written.

**Reason:** The app has no window for diagnostics, and the unified log is not readable in every environment. The real-app checks of 2026-11-17 used it.
