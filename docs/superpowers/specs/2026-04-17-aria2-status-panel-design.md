# Aria2 Status Panel Design

**Date:** 2026-04-17

## Goal

Add a persistent Aria2 status area inside the Settings page so users can see current engine state, the most recent engine error, and recovery actions without relying on transient toast messages.

## Scope

- Only the Settings page Aria2 management area changes.
- No global banner changes.
- No backend protocol additions for this iteration.

## Design

### State model

The frontend will keep a small Aria2 diagnostics state with:

- `connectionState`
- `engineReady`
- `isRestarting`
- `lastError`
- `lastErrorAt`
- `lastSuccessAt`

This state will be updated from two sources:

1. Existing engine lifecycle events: `aria2-ready` and `aria2-connection`
2. Manual restart actions from the Settings page

### Behavior

- Restart failures are persisted in the diagnostics state and shown in the Settings page until the next successful engine connection.
- `connected` and `aria2-ready` clear the previous restart error and record a success timestamp.
- `disconnected` and `reconnecting` update visible state but do not overwrite a more specific recorded error.
- The Settings page exposes a single primary action:
  - `Restart Engine` when a config restart is required
  - `Retry Connection` otherwise

### UI

Add an Aria2 status card at the top of the Developer section showing:

- current state badge
- current RPC port
- most recent success time
- most recent error and timestamp
- restart / retry action

Toast messages remain as supplemental feedback, not the primary source of truth.

## Testing

- Add a pure TypeScript diagnostics reducer module.
- Add a Node test that exercises restart failure, reconnecting, and recovery transitions.
- Run the targeted Node test and the existing Rust tests after wiring the UI.
