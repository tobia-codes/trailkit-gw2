# trailkit-ui-testing

Test tools for trailkit's ImGui code (memo plan M65). A **dev-dependency
only** — it is never compiled into an addon DLL.

| Module | Content | Platform |
|--------|---------|----------|
| `raster` | CPU rasterizer for ImGui draw data (textured, vertex-colored triangles, top-left fill rule, straight-alpha blending) | any |
| `snapshot` | `assert_snapshot` — compares an image with `tests/snapshots/<name>.png` (tolerance: 3/255 per channel, no differing pixels); writes `tests/snapshots/failures/<name>.actual.png` + `.diff.png` on failure | any |
| `harness` | `UiHarness` — headless ImGui context with the real trailkit fonts (installed for `FontScope`), Nexus-like key map, simulated mouse/keyboard input per frame, rendering to images | Windows |

## Usage

```rust
let mut harness = UiHarness::new([360.0, 180.0]);
let image = harness.settle(3, &FrameInput::default(), |ui| {
    // draw a window with the code under test
});
assert_snapshot(env!("CARGO_MANIFEST_DIR"), "editor-dark", &image);
```

Tests that use the harness must link the same `trailkit-ui` as the harness:
in `trailkit-ui` itself they are integration tests (`tests/*.rs`); in addons
they can be unit tests.

## Running

From WSL (the harness needs nexus-rs' ImGui, which is Windows only):

```sh
cargo xtask ui-test            # build with cargo-xwin, run the .exe files via interop
cargo xtask ui-test --update   # accept the current rendering as new references
```

On Windows (CI) the tests run with a plain `cargo test --workspace`.
Rendered images are not pixel-identical to GW2's DirectX output, but layout,
sizes and colors come straight from ImGui. Nexus-specific behavior (keyboard
routing, Escape, the shared font atlas) still needs an in-game test.
