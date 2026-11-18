# trailkit-nexus-bridge

The Nexus API helpers every trailkit addon shares (ROADMAP R132, R136, Q496).
It is the only shared crate that calls the Nexus API: each addon talks to
Nexus only from its own `src/nexus_bridge` module, and what several addons
need from Nexus lives here. [`trailkit-ui`](../trailkit-ui/README.md) and the
addons' windows and slices only draw with ImGui and know nothing of Nexus, so
a port to another host replaces only the bridges.

> **Status:** used by `trailkit-memo`. Windows only: on other targets the crate
> is empty, like every Nexus binding (ROADMAP R39).

| Module | Content | Platform |
|--------|---------|----------|
| `fonts` | `register_fonts`: registers the shared glyph texts and fonts with Nexus and hands every delivered font to `trailkit-ui`'s font registry; `follow_body_size`: resizes them when Nexus' `FontUI` changes | Windows |

## Packages

The packages this project uses directly.

| Package | Type | What it does | Why it is used here |
|---------|------|--------------|---------------------|
| [`trailkit-ui`](../trailkit-ui/README.md) | own, Windows only | Theme, fonts and shared ImGui components. | Provides the font roles and receives the loaded fonts. |
| `nexus` | external, Windows only | Rust bindings for the Raidcore Nexus addon API and its ImGui. | Registers and resizes fonts through Nexus. |

## Build and test

```sh
cargo xwin clippy --target x86_64-pc-windows-msvc -p trailkit-nexus-bridge -- -D warnings
```

The crate has no tests of its own: it only forwards to Nexus, and memo's
in-game tests cover the fonts.
