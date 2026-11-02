# Source Cutter

A 3D modelling tool built for the Source Engine: Hammer units, a Hammer-style grid
and a Hammer-inspired workflow, with Blender-like editing (modes, vertex/edge/face).

It is **not** a Hammer replacement: the goal is to model geometry for Source with a
workflow that feels familiar to mappers, without Blender's generic setup.

## Run

```sh
cargo run            # debug
cargo run --release  # smoother
cargo test
```

## Controls

### Navigation

| Action                    | Input                                          |
|---------------------------|------------------------------------------------|
| Orbit                     | Middle mouse drag, or drag the compass         |
| Pan                       | Shift + middle mouse drag                      |
| Zoom                      | Mouse wheel                                    |
| Top / front / side view   | Click an axis on the compass (orthographic)    |
| Perspective / ortho       | `5`, or the button under the compass           |
| Frame selection           | `F`                                            |

### Modes and selection (Blender-like)

| Action                       | Input                          |
|------------------------------|--------------------------------|
| Object / Edit Mode           | `Tab`, or top-left selector    |
| Vertex / edge / face select  | `1` / `2` / `3` (Edit Mode)    |
| Select / add to selection    | Click / Shift+click            |
| Select all / deselect all    | `A` / `Alt+A`                  |
| Delete                       | `Del` or `X`                   |
| Undo / redo                  | `Ctrl+Z` / `Ctrl+Shift+Z`, `Ctrl+Y` |

### Tools

| Tool    | Key       | Usage                                                         |
|---------|-----------|---------------------------------------------------------------|
| Select  | `Shift+S` | Click to select                                               |
| Move    | `G`       | Drag an arrow (axis), a square (plane) or the circle (grid plane) |
| Scale   | `S`       | Drag a side handle; the opposite side stays fixed (Hammer)    |
| Block   | `Shift+B` | Drag on the grid to create a box                              |

Grid size: `[` / `]` (or `-` / `+`). Moves and resizes snap to the grid. `Esc` cancels.

## Project structure

```
src/
├── main.rs              Entry point
├── app.rs               Panel layout and global shortcuts
├── units.rs             Source conventions (units, grid, player hull, map limits)
├── math/                Geometry: Aabb, Ray (picking, axis dragging)
├── scene/               The document (everything here is undoable)
│   ├── mesh.rs          Polygon mesh: vertices, faces, edges, raycast, delete
│   ├── object.rs        Named mesh object
│   └── selection.rs     Object selection + vertex/edge/face selection
├── editor/              Editor state
│   ├── mod.rs           Editor: modes (Object/Edit), actions
│   ├── camera.rs        Orbit camera, perspective / orthographic views
│   ├── grid.rs          Grid size, snapping, work plane (XY / XZ / YZ)
│   └── history.rs       Undo / redo (scene snapshots)
├── tools/               Interactive tools (update + their own overlay drawing)
│   ├── select.rs        Object and element picking
│   ├── move_tool.rs     Move gizmo
│   ├── scale_tool.rs    Hammer-style resize handles
│   ├── block.rs         Block creation
│   └── transform.rs     Shared: transformed vertices, bounds
├── viewport/            3D view
│   ├── mod.rs           Mouse input → camera / tools, then drawing
│   ├── projector.rs     World ↔ screen projection, near-plane clipping
│   ├── draw_grid.rs     Hammer grid (1024 / 64 / grid lines, map bounds)
│   ├── draw_scene.rs    Objects, Edit Mode overlay, previews
│   ├── visibility.rs    Occlusion tests for Edit Mode
│   └── nav_gizmo.rs     Blender-style compass
└── ui/                  Menus, toolbar (mode selector), tool strip, properties, status bar, theme
```

### Adding a tool

1. Create `src/tools/my_tool.rs` with `update(&mut self, input: &ToolInput, scene, history)`
   and, if needed, `draw(&self, painter, proj, scene, edit)`.
2. Add a variant to `ToolKind` (label, icon, shortcut) and a field in `Tools`.
3. Call `history.checkpoint(scene)` before modifying the scene so it can be undone.
4. Add its settings in `ui/properties.rs`.
