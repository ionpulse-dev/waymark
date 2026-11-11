# Decisions: Web console

## A web console replaces the macOS app — 2026-11-11

**Decision:** The jobs of the old app that need a human channel (approval, activity, undo, review, pause) go to a web console served by the server.

**Alternatives considered:**
- A native app for each system. High cost, and macOS only at first.
- Nothing. Then no enforced approval exists.

**Reason:** A web page works on every system, needs no install, and is a channel that the agent cannot reach.

## Server mode only — 2026-11-11

**Decision:** Local mode has no console.

**Reason:** Local mode runs for one session and has no process that serves a page. The client's prompts and the personal config cover the jobs there.
