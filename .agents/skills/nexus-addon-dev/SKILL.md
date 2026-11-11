---
name: nexus-addon-dev
description: Project knowledge for building Raidcore Nexus addons in Rust for trailkit-gw2 — Nexus API facts, threading and performance rules, inter-addon communication, settings window, localization, help page, API key handling. Use whenever writing, reviewing or planning code under addons/ or crates/ that runs inside Guild Wars 2.
---

# Nexus addon development (trailkit-gw2)

Facts below were researched on 2026-10-31 against Nexus API v6
(`RCGG-lib-nexus-api/Nexus.h`, Nexus source, `zerthox/nexus-rs` 0.12).
Re-verify when bumping `NEXUS_API_VERSION` or `nexus-rs`.

## 0. Addon structure (ROADMAP R76)

Practical template with directory tree, slice contract and a checklist
for new addons: [`references/addon-structure.md`](references/addon-structure.md).

- An addon = app crate (`addons/<addon>/`, builds the DLL, wires slices)
  + slice crates (`addons/<addon>/slices/<slice>/`, crate
  `trailkit-<addon>-<slice>`) + `trailkit-<addon>-shared`.
- A slice **never** depends on another slice; shared needs go to the
  shared crate (addon-internal) or `crates/` (cross-addon). The app passes
  neutral shared types between slices.
- A slice owns its state, `draw` function, text enum and help section;
  removing a feature = removing its folder, dependency and wiring.
- **Slice convention** (a convention, not a trait — inputs differ per
  slice and the app calls each slice directly):
  - state struct owned by the slice, held by the app as a field, drawn
    with `draw(&mut self, ui, language, theme, …)`;
  - returns `Option<XyzAction>` (one user action per frame) or a small
    output struct when more comes back (e.g. action + "UI state
    changed"); the app maps it to its own `WindowAction`;
  - never touches the storage worker: persisted values come in as
    `&mut` to the single fields it needs (not the whole `UiState`), the
    app saves when the slice reports a change;
  - texts: one enum implementing `trailkit_<addon>_shared::i18n::LocalizedText`
    (`ALL`, `get`, optional `extra_characters`); its test calls
    `assert_all_translated::<T>()`; the app's `all_texts()` concatenates
    every set for the font atlas.

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
- Update timing (verified in Nexus source v2026.11.8.830): Nexus checks for an
  update only when it loads an addon (game start or manual load), never
  periodically; when one is found it hot-reloads the addon without a game
  restart, unless the addon lacks `unload`, sets `DisableHotloading` or
  `OnlyLoadDuringGameLaunchSequence`. (Nexus `main` is an unreleased rewrite
  that checks hourly but only swaps the file; recheck on its release.)
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
  always reach ImGui. `CaptureKeyboardFromApp` is not enough. Shortcuts vary and
  the reason is unknown (tried in game 2026-11-06, no input active): Ctrl+Z, Ctrl+Y
  and Ctrl+0 reached ImGui, Ctrl+F and Delete did not, while tests pass either way.
  Try each new key in game. One that does not arrive uses a `WndProc` callback that
  flips an atomic and consumes the key while it belongs to the window (memo M174,
  M175). A click into the game world takes the keyboard from ImGui without being
  reported as a click, so never reactivate an input that lost it (memo M178). A
  custom text
  widget keeps an invisible ImGui text input active while focused
  (`trailkit_ui::editor::view`, "keyboard sink"), reads
  `InputQueueCharacters` before that input consumes them, and writes the
  clipboard after it. The sink must stay **in view** (draw it at the
  scrolled top): ImGui does not activate an input scrolled out of its
  window, and then every key goes to the game (memo M83).
- ImGui is **1.80**: no `BeginDisabled` (draw disabled items in
  `text.disabled` and ignore clicks), no line wrapping in multiline inputs,
  and Escape in an active input reverts it to its content at activation.
- Paths: `Paths_GetAddonDirectory("trailkit")` wrapped in
  `trailkit_storage::TrailkitDirectory` (R92): `addon_data(addon)` for settings
  and caches, `logs()`, `shared()`; never write next to the game exe.
- Logging (R92): nexus-rs' `log` feature is off; `trailkit_log::start` is the
  first call in `load` and its `LogSession` is dropped last in `unload`. It
  forwards WARN/ERROR to `Nexus.log` itself.

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
   panic hook; `trailkit-log` chains its own in front (writes a `panic` line).
   Worker threads catch and log errors, never `unwrap()`.

## 3. Inter-addon communication

Nexus offers two primitives:

- **DataLink** (`DataLink_Share(id, size)` / `DataLink_Get(id)`): a named,
  fixed-size shared memory block owned by the sharer.
- **Events** (`Events_Raise(id, ptr)`, targeted variants): synchronous
  callbacks, payload is a raw pointer.

Project contract (to be finalized in `docs/plans/addons/trailkit-core.md`):

- **Type layers (ROADMAP R75):** domain types (what an event, waypoint,
  chat link, note summary or language *is* at Trailkit) live in
  `crates/trailkit-model` (serde, no Nexus/ImGui) and are used by every
  addon, datagen and the backend; GW2 API DTOs stay at the boundary
  (datagen, core) and are refined into model types via `TryFrom`; addons
  never see the API shape. An addon owns its own data (memo: notes) and
  shares only slim model types through the protocol.
- All cross-addon FFI types live in **one** crate (`crates/trailkit-protocol`),
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
  Write one paragraph or list item per source line: single line breaks are
  shown as line breaks (memo M63).
  Use only characters the subset fonts contain (Latin up to Extended-B,
  general punctuation, arrows): keep writing menu paths as
  `Nexus › Options` (a test checks every character, M62).
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

## 8. Facts learned in game

**Nexus**

- It consumes Escape before ImGui: always use `GUI_RegisterCloseOnEscape`.
- It keeps fonts per identifier until the game restarts, so a font change needs a
  restart, not a hot reload. `resize_font` flickers: debounce it.
- "Loaded addon" appears in `Nexus.log` 1-2 s after a deploy returns. Check again
  before calling a reload failed. Its directory watcher once stopped firing for a
  whole game session (Nexus 2026.10.25.60, 2026-11-04): rename, copy and PowerShell
  all went unseen, while manual unload/load worked and a game restart fixed it. If it
  persists, ask in the Raidcore Discord.
- The game's hover cursor (gear, speech bubble) leaks through addon windows: a Nexus
  issue.
- Font sizes differ in game, unlike in tests. Measured 2026-11-06 at the user's text size:
  body and icons draw at 20 px, and a frame is 26.75 px high in the window's font but
  28 px in the icon font.

**ImGui 1.80**

- A child's background goes into the parent's draw list, below siblings drawn earlier.
  Overlays draw into their own list (`draw_background(false)`). A borderless child has
  no window padding and is not clipped at its edge.
- `AddLine` already adds half a pixel. Add none yourself, or a 1 px line shows as a dim
  2 px band (not in the test renderer).
- `Selectable` and `MenuItem` highlights have radius 0, and their item rect reaches half
  the item spacing outwards. Disabled items draw their label in `TextDisabled`;
  `IsItemHovered` is true for them once the item has ended.
- A tree node's triangle takes the text color: push a transparent `Text` while pushing
  the node, then draw chevron and title yourself.
- `IsWindowHovered` is false while any item is active: use
  `ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM`. `is_mouse_hovering_rect` ignores windows above
  it: combine it with the item's hover.
- Alpha 0 windows skip all items. `CalcTextSize` rounds up per call: measure glyphs with
  `ImFont_CalcTextSizeA`.
- Dragging where no item is moves the window: `movable(false)` on children. Windows with
  `NO_TITLE_BAR` move from any empty area even with `ConfigWindowsMoveFromTitleBarOnly`.
- Submenus overlap their parent by `ItemInnerSpacing.x`; `menu::begin_submenu` sets it to 0.
- A tooltip sits at mouse + 24 px × `style.MouseCursorScale`; `StyleScope` sets the scale
  from the measured cursor (`cursor_extent`).
- `DrawListMut` is a singleton in imgui-rs: shared components draw through the raw
  `ImDrawList_*` calls when a caller may hold one.

**Fonts and icons**

- Open Sans sits low in ImGui's line box: center labels by their capitals
  (`glyph_metrics::capital_middle`, `draw_button`, `draw_tooltip`).
- Center icons with `glyph_icon_ink` and badge letters with `glyph_visible_ink`. Its 50 %
  threshold misses thin strokes that straddle two pixel rows.
- The Lucide source font stores `xMin`/`yMin` = 0 for every glyph; `cargo xtask
  subset-fonts` rewrites the boxes from the outline points (`fix_glyph_bounds`).
- A codepoint by name: parse the source TTF's `cmap` and `post` tables with a stdlib
  script (no fontTools here).
- Characters in help pages must exist in the subset fonts; the test
  `every_character_is_in_the_subset_fonts` catches it.
- Anything sized from an icon button's natural width can overflow in game while tests
  look fine: size inner controls from the ink or a fixed square.

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
