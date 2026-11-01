---
name: nexus-addon-dev
description: Project knowledge for building Raidcore Nexus addons in Rust for trailkit-gw2 — Nexus API facts, threading and performance rules, inter-addon communication, settings window, localization, help page, API key handling. Use whenever writing, reviewing or planning code under addons/ or crates/ that runs inside Guild Wars 2.
---

# Nexus addon development (trailkit-gw2)

Facts below were researched on 2026-10-31 against Nexus API v6
(`RCGG-lib-nexus-api/Nexus.h`, Nexus source, `zerthox/nexus-rs` 0.12).
Re-verify when bumping `NEXUS_API_VERSION` or `nexus-rs`.

## 1. Platform facts

- An addon is a Windows DLL exporting `GetAddonDef()`. In Rust: the
  `nexus::export!` macro from [`nexus-rs`](https://github.com/zerthox/nexus-rs)
  (MIT, git dependency, ImGui via `arcdps-imgui` 0.8 = imgui-rs 0.8 / older
  Dear ImGui: no docking branch, tables API available).
- Target: `x86_64-pc-windows-msvc`, `crate-type = ["cdylib"]`.
- `signature`: positive ID assigned by Raidcore when listed; use a random
  negative value until then.
- Update providers (verified in Nexus source v2026.10.25): `GitHub` lists
  `/repos/<repo>/releases`, accepts tags matching `v?X.Y(.Z(.W))`, and picks
  the first `.dll` asset of the highest version — unusable in our monorepo.
  `Raidcore` is unimplemented. `Direct` compares the MD5 at `<update_link>.md5`
  (fallback `.md5sum`) with the local DLL. **We use `Direct`** (ROADMAP R49).
- Hot reload: Nexus can unload/reload an addon at runtime. `unload` **must**
  stop and join every thread, deregister every callback/keybind/event and drop
  every DataLink pointer — otherwise the game crashes after unload.
  `AF_DisableHotloading` exists but do not use it unless unavoidable.
- One Dear ImGui context is **shared by all addons**. Never change the global
  style; always push style colors/vars before drawing and pop them after
  (use the `trailkit-ui` theme scope guard).
- Fonts: `NexusLinkData_t` exposes `Font`, `FontBig`, `FontUI`;
  `Fonts_AddFromFile/Memory` + `Fonts_Resize` exist for custom sizes.
  Font changes rebuild the atlas — never do it per frame.
  Verified in Nexus' `FontManager.cpp` (2026-11): `Fonts_AddFromMemory`
  **copies** data and config (embedded bytes are fine); adding an existing
  identifier only adds a callback (fonts are shared between addons); callbacks
  get the new `ImFont*` after every atlas rebuild. The atlas only holds Basic
  Latin, Latin-1, Latin Extended-A/B **plus every character of every
  registered localization text** — register icon glyphs and the addon's own
  UI strings as localization texts *before* adding fonts, or they render as
  `?`. All trailkit addons use `trailkit_ui::font_registry` for this.
- `NexusLinkData_t.IsGameplay` / `IsMoving` / `IsCameraMoving` tell you if the
  player is in gameplay (hide/skip work during loading screens & cutscenes).
  **Project rule (R56/R61):** windows are not drawn while `IsGameplay` is
  false (unless the addon's "also show outside gameplay" setting is on); the
  open state is kept. Cache the `NexusLink` pointer in `load`, read the flag
  after the cheap `VISIBLE` check, drop the pointer in `unload`.
- Options: `RT_OptionsRender` draws into the Nexus addon options tab.
  **Project rule (ROADMAP R53):** only a header (addon name + version) and
  the buttons "Open settings" and "Open help"; real settings live in the
  addon's own settings window.
- Quick access: `QuickAccess_Add(identifier, texture, hoverTexture, keybind,
  tooltip)` adds an icon to the Nexus bar bound to a keybind (a click
  triggers the keybind). Textures come from committed PNGs embedded with
  `include_bytes!` and loaded via `Textures_LoadFromMemory` (R57).
- Keybinds: `InputBinds_RegisterWithString` — the user can rebind in Nexus.
- Close on Escape: `GUI_RegisterCloseOnEscape(windowName, &visible)` — Nexus
  then writes the bool directly. **Nexus consumes Escape before ImGui sees
  it** (verified in game 2026-11-01: an addon's own `IsKeyPressed(Escape)`
  check never fires), so always register. A window that must ask before
  closing (unsaved changes) notices the cleared flag in its render callback,
  shows itself again and asks (memo M54). Deregister in `unload`.
- **Keyboard routing (verified 2026-11-01):** Nexus passes key presses
  (arrows, Backspace, Enter, …) to ImGui instead of the game only while a
  real ImGui text input is active (`io.WantTextInput`); typed characters
  always reach ImGui. `CaptureKeyboardFromApp` is not enough. A custom text
  widget keeps an invisible ImGui text input active while focused
  (`trailkit_ui::editor::view`, "keyboard sink"), reads
  `InputQueueCharacters` before that input consumes them, and writes the
  clipboard after it.
- ImGui is **1.80**: no `BeginDisabled` (draw disabled items in
  `text.disabled` and ignore clicks), no line wrapping in multiline inputs,
  and Escape in an active input reverts it to its content at activation.
- Paths: `Paths_GetAddonDirectory("trailkit/<addon>")` for settings and
  caches; never write next to the game exe.

## 2. Threading & performance (mandatory)

Threads that touch the game:

| Thread | Who calls | Allowed work |
|--------|-----------|--------------|
| Render thread | `RT_PreRender` / `RT_Render` / `RT_PostRender` / `RT_OptionsRender` every frame | ImGui only. Read pre-computed state. No I/O, no locks that can block, no allocations in steady state, no parsing. |
| Event raiser's thread | `Events_Subscribe` callbacks run **synchronously on whatever thread called `Events_Raise`**, while Nexus holds a global recursive mutex | Copy the payload (pointer is only valid during the callback), push it into a channel/atomic, return. Never block, never raise another event that could wait on you. |
| Keybind callback | Nexus input thread (WndProc) | Flip an atomic / send a message. Nothing else. |
| Own worker thread(s) | spawned in `load`, joined in `unload` | Network, disk, parsing, cache, heavy computation. |

Rules:

1. **Sleeping when closed.** When no window of the addon is visible the render
   callback returns after one relaxed atomic load (`if !VISIBLE.load() {
   return; }`). No other per-frame work. UI-only addons (memo, event) have no
   worker thread running while closed except to service explicit messages
   (blocked on `recv()`, i.e. zero CPU).
2. **Worker threads block, never spin.** Use `std::sync::mpsc` /
   `crossbeam-channel` `recv()` or `recv_timeout(next_due)`. No `loop {
   sleep(10ms) }` polling.
3. **One worker per addon** by default (core: one network/scheduler thread).
   No Tokio runtime inside addons — use blocking `ureq` HTTP on the worker
   thread; Tokio is for the data pipeline / future backend only.
4. **Render reads snapshots, not locks.** Worker builds an immutable snapshot
   and publishes it via `arc-swap` (`ArcSwap<Snapshot>`); render does
   `load()` (lock-free). Never hold a `Mutex` across ImGui drawing.
5. **Precompute strings and layout** on the worker (formatted times, sorted
   lists, translated labels cached per language change).
6. **Time-dependent UI** (countdowns) recomputes at most once per second, not
   per frame.
7. **Background work is invisible to the player**: throttle network bursts,
   lower worker thread priority (`THREAD_PRIORITY_BELOW_NORMAL`), skip
   refreshes during loading screens if not needed, never do I/O on the render
   thread — even "small" file reads.
8. **Panics**: a panic must never cross the FFI boundary. nexus-rs installs a
   panic hook; worker threads catch and log errors, never `unwrap()`.

## 3. Inter-addon communication

Nexus offers two primitives:

- **DataLink** (`DataLink_Share(id, size)` / `DataLink_Get(id)`): a named,
  fixed-size shared memory block owned by the sharer.
- **Events** (`Events_Raise(id, ptr)`, targeted variants): synchronous
  callbacks, payload is a raw pointer.

Project contract (to be finalized in `docs/plans/addons/trailkit-core.md`):

- All cross-addon types live in **one** crate (`crates/trailkit-protocol`),
  `#[repr(C)]`, with an explicit `version: u32` as first field. No Rust-ABI
  types (`String`, `Vec`, `Box`, trait objects) cross the DLL boundary.
- Memory is freed by the DLL that allocated it.
- All `unsafe` FFI glue lives in that crate; addon code only sees safe, typed
  wrappers (e.g. `CoreClient::events() -> Option<EventSnapshot>`).
- Consumers subscribe to `EV_ADDON_LOADED` / `EV_ADDON_UNLOADED` and
  re-acquire / drop core pointers when core (re)loads. Core being absent is a
  normal state: features grey out and show a hint (see §5).

## 4. Localization

- Every addon ships **English and German**. Strings live in typed keys
  (an enum / generated module per addon, shared keys in `trailkit-i18n`),
  never as inline literals in UI code.
- Nexus does **not** expose the active language directly. Trick: at load,
  `Localization_Set("trailkit.shared.localization.language", "en", "en")`,
  `Localization_Set("trailkit.shared.localization.language", "de", "de")` etc.; then
  `Localization_Translate("trailkit.shared.localization.language")` returns the active Nexus language
  code.
- Resolution order: language chosen in the addon settings → Nexus language →
  English.

## 5. UX contracts shared by all addons

- **Own settings window** per addon (opened from: the Nexus options button,
  the addon's main window menu, and optionally a keybind).
- **Help page** per addon (window/tab "Help"), content in both languages,
  generated from `addons/<name>/help/{en,de}.md`. Any change to user-visible
  behavior must update the help page in the same change.
- **Missing dependency guidance**: if a feature needs data the user disabled
  in core (or core is not installed), the feature is greyed out and shows an
  inline hint explaining *what* to enable *where*, with a button that opens
  the relevant core settings page (or the Nexus addon library for install).
- Look & feel: follow the `trailkit-ui` skill.

## 6. Player API keys

- The GW2 API key is stored **only** by trailkit-core, encrypted at rest with
  Windows DPAPI (`CryptProtectData`, current-user scope). Never in plain text,
  never in logs, never sent anywhere except `https://api.guildwars2.com`.
- Never sent to trailkit data hosts or any future trailkit backend.
- Validate with `/v2/tokeninfo`; show granted permissions; mask in UI
  (show/hide toggle, paste button, clear button).
- Other addons never receive the key — they request data through core.

## 7. Build & deploy (dev)

- WSL cross-compile with `cargo-xwin` (ROADMAP R7, R28–R32); needs LLVM
  (`clang`, `lld`, `llvm`) because `arcdps-imgui-sys` builds Dear ImGui with
  `clang-cl`.
- Build + copy into the game: `cargo xtask deploy <addon> [--release]`
  (folder from `TRAILKIT_GW2_ADDONS_DIR` or `xtask.local.toml`).
- Hot reload verified in game (2026-10-31): Windows refuses to overwrite a
  loaded DLL, so deploy renames it to `<name>.dll.old` first (Nexus' own
  update technique) and moves the new DLL in; Nexus detects the change,
  unloads (~0.3 ms) and loads the new build, and later removes the `.old`
  file itself. Keybinds and state registered in `load` must therefore be
  fully reverted in `unload` (`revert_on_unload`).
- Crate layout: `crate-type = ["cdylib", "rlib"]`, `nexus` as a
  `[target.'cfg(windows)'.dependencies]` entry, Nexus glue in a
  `#[cfg(windows)] mod addon;` → logic modules are tested on Linux with
  `cargo test`, Windows code linted with `cargo xwin clippy --target
  x86_64-pc-windows-msvc` (R39).
- Keybinds that must be unbound by default are registered with the string
  `"(null)"`. Keybind display names are set via `set_translation(<keybind id>,
  <lang>, <text>)`.

## Persisted user data (ROADMAP R22)

- Every file an addon writes (notes, custom events, settings, theme) contains
  a `schema_version`.
- On load, older versions are migrated forward step by step; **write a backup
  copy before the first migrated save**.
- A file with an unknown *newer* `schema_version` is never overwritten: run
  read-only (or disabled) and tell the user to update the addon.
- Saves are atomic (write temp file + rename) on the worker thread.
- Use `crates/trailkit-storage` for all of the above — never hand-roll file
  persistence in an addon (memo plan M29).
- Safety copies (migration copies, copies of unreadable files, backups) go
  into one `backups/` subfolder with a fixed cap per kind and automatic
  pruning, so the addon folder never fills up with forgotten copies
  (memo plan M33). An unreadable file is never overwritten.
