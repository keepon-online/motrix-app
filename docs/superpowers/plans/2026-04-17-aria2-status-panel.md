# Aria2 Status Panel Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a persistent Aria2 status panel to the Settings page that shows current engine status, the latest restart error, and retry/restart actions.

**Architecture:** Extract Aria2 diagnostics transitions into a small pure frontend module, then consume that state from a shared composable used by the Settings page. Keep backend APIs unchanged and layer the new UX on top of existing `aria2-ready` and `aria2-connection` events.

**Tech Stack:** Vue 3, TypeScript, Element Plus, Tauri events, Node built-in test runner, esbuild

---

## Chunk 1: Diagnostics State

### Task 1: Add the failing diagnostics transition test

**Files:**
- Create: `tests/aria2-diagnostics.test.mjs`
- Test: `tests/aria2-diagnostics.test.mjs`

- [ ] **Step 1: Write the failing test**
- [ ] **Step 2: Run `node --test tests/aria2-diagnostics.test.mjs` and verify it fails for the missing diagnostics module**
- [ ] **Step 3: Implement the minimal diagnostics transition module**
- [ ] **Step 4: Run `node --test tests/aria2-diagnostics.test.mjs` and verify it passes**

### Task 2: Expose shared Aria2 diagnostics state

**Files:**
- Create: `src-vue/composables/useAria2Diagnostics.ts`
- Modify: `src-vue/composables/useConnectionStatus.ts`
- Test: `tests/aria2-diagnostics.test.mjs`

- [ ] **Step 1: Add shared refs and event-driven updates**
- [ ] **Step 2: Keep existing connection-status consumers working**
- [ ] **Step 3: Re-run `node --test tests/aria2-diagnostics.test.mjs`**

## Chunk 2: Settings UI

### Task 3: Render the persistent Aria2 status panel in Settings

**Files:**
- Modify: `src-vue/views/Settings.vue`
- Modify: `src-vue/locales/en.ts`
- Modify: `src-vue/locales/zh-CN.ts`

- [ ] **Step 1: Add panel state bindings and restart action wiring**
- [ ] **Step 2: Render status, latest error, timestamps, and retry/restart CTA**
- [ ] **Step 3: Add locale copy and scoped styles**

### Task 4: Verify the integrated change

**Files:**
- Verify: `tests/aria2-diagnostics.test.mjs`
- Verify: `src-vue/views/Settings.vue`

- [ ] **Step 1: Run `node --test tests/aria2-diagnostics.test.mjs`**
- [ ] **Step 2: Run `cargo test` in `src-tauri`**
- [ ] **Step 3: Run `npm run build`**
