# Source Cutter

A 3D modelling tool built for the Source Engine (Hammer units, Hammer-style grid).

## Run

```sh
cargo run            # debug
cargo run --release  # smoother
cargo test
```

## Controls

| Action                 | Input                                      |
|------------------------|--------------------------------------------|
| Orbit                  | Middle mouse drag, or drag the compass     |
| Pan                    | Shift + middle mouse drag                  |
| Zoom                   | Mouse wheel                                |
| Look from an axis      | Click an axis on the compass (top right)   |
| Select tool            | Shift+S                                    |
| Block tool             | Shift+B, then drag on the grid             |
| Grid size              | `[` / `]` (or `-` / `+`)                   |
| Delete selection       | Del                                        |
| Frame selection        | F                                          |
| Cancel / deselect      | Esc                                        |

## Project structure

```
src/
├── main.rs              Entry point
├── app.rs               Panel layout and global shortcuts
├── units.rs             Source conventions (units, grid, player hull, map limits)
├── math/                Geometry: Aabb, Ray (picking)
├── scene/               The document: Scene (brushes + selection), Brush
├── editor/              Editor state: Editor, OrbitCamera, GridSettings
├── tools/               Tools (independent of egui): Select, Block
├── viewport/            3D view
│   ├── mod.rs           Mouse input → camera / tools, then drawing
│   ├── projector.rs     World ↔ screen projection, near-plane clipping
│   ├── draw_grid.rs     Hammer grid (1024 / 64 / grid lines, map bounds)
│   ├── draw_scene.rs    Brushes, selection, Block tool preview, player
│   └── nav_gizmo.rs     Blender-style compass
└── ui/                  Menus, toolbar, tool strip, properties, status bar, theme
```

### Adding a tool

1. Create `src/tools/my_tool.rs` with an `update(&mut self, input: &ToolInput, scene: &mut Scene)` method.
2. Add a variant to `ToolKind` (label, icon, shortcut) and a field in `Tools`.
3. Add its settings in `ui/properties.rs` and its preview drawing in `viewport/`.
