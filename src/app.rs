//! Application shell: panel layout and global keyboard shortcuts.

use eframe::egui::{self, Key, KeyboardShortcut, Modifiers};

use crate::editor::Editor;
use crate::scene::SelectMode;
use crate::tools::ToolKind;
use crate::{ui, viewport};

pub struct SourceCutterApp {
    editor: Editor,
    icons: ui::icons::Icons,
    /// Tab was pressed this frame (taken from egui, see `raw_input_hook`).
    tab_pressed: bool,
}

impl SourceCutterApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        ui::theme::apply(&cc.egui_ctx);
        Self { editor: Editor::default(), icons: ui::icons::Icons::load(&cc.egui_ctx), tab_pressed: false }
    }
}

impl eframe::App for SourceCutterApp {
    /// egui uses Tab to move keyboard focus between buttons, which breaks the
    /// Blender-style Tab shortcut. Outside of text fields, Tab is removed from egui's
    /// input and handled by the editor instead.
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        if ctx.text_edit_focused() {
            return;
        }
        raw_input.events.retain(|event| match event {
            egui::Event::Key { key: Key::Tab, pressed, repeat, .. } => {
                self.tab_pressed |= *pressed && !*repeat;
                false
            }
            _ => true,
        });
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if std::mem::take(&mut self.tab_pressed) {
            self.editor.toggle_mode();
        }
        handle_shortcuts(ui.ctx(), &mut self.editor);
        self.editor.validate();

        let editor = &mut self.editor;
        let dark_frame = egui::Frame::side_top_panel(ui.style()).fill(ui::theme::PANEL_DARK);

        egui::Panel::top("menu_bar").show(ui, |ui| ui::menu_bar::show(ui, editor));

        egui::Panel::bottom("status_bar")
            .frame(dark_frame)
            .show(ui, |ui| ui::status_bar::show(ui, editor));

        egui::Panel::left("tool_strip")
            .resizable(false)
            .exact_size(46.0)
            .frame(dark_frame)
            .show(ui, |ui| ui::tool_strip::show(ui, editor, &self.icons));

        egui::Panel::right("properties")
            .default_size(270.0)
            .min_size(220.0)
            .show(ui, |ui| ui::properties::show(ui, editor));

        egui::Panel::top("toolbar").show(ui, |ui| ui::toolbar::show(ui, editor));

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| viewport::show(ui, editor));
    }
}

fn handle_shortcuts(ctx: &egui::Context, editor: &mut Editor) {
    // Don't steal keys while typing in a text field.
    if ctx.text_edit_focused() {
        return;
    }

    ctx.input_mut(|i| {
        // Undo / redo (Ctrl+Shift+Z must be checked before Ctrl+Z).
        if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z))
            || i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::Y))
        {
            editor.redo();
        }
        if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::Z)) {
            editor.undo();
        }
        if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::N)) {
            editor.new_scene();
        }
        if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::SHIFT, Key::S)) {
            editor.set_tool(ToolKind::Select);
        }
        if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::SHIFT, Key::B)) {
            editor.set_tool(ToolKind::Block);
        }
        if i.key_pressed(Key::G) && i.modifiers.is_none() {
            editor.set_tool(ToolKind::Move);
        }
        if i.key_pressed(Key::S) && i.modifiers.is_none() {
            editor.set_tool(ToolKind::Scale);
        }
        // Grid size: [ ] like Hammer, plus - / + for keyboards where brackets are awkward (AZERTY).
        if i.key_pressed(Key::OpenBracket) || i.key_pressed(Key::Minus) {
            editor.grid.decrease();
        }
        if i.key_pressed(Key::CloseBracket) || i.key_pressed(Key::Plus) || i.key_pressed(Key::Equals) {
            editor.grid.increase();
        }
        if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::ALT, Key::A)) {
            editor.deselect_all();
        }
        if i.key_pressed(Key::A) && i.modifiers.is_none() {
            editor.select_all();
        }
        if i.key_pressed(Key::Delete) || (i.key_pressed(Key::X) && i.modifiers.is_none()) {
            editor.delete_selected();
        }
        if i.key_pressed(Key::Escape) {
            editor.escape();
        }
        // 1 / 2 / 3: vertex / edge / face select mode (Edit Mode). Tab is handled in `ui`.
        if editor.edit_context().is_some() && i.modifiers.is_none() {
            for (key, mode) in [(Key::Num1, SelectMode::Vertex), (Key::Num2, SelectMode::Edge), (Key::Num3, SelectMode::Face)] {
                if i.key_pressed(key) {
                    editor.set_select_mode(mode);
                }
            }
        }
        if i.key_pressed(Key::Num5) && i.modifiers.is_none() {
            editor.camera.toggle_projection();
        }
        if i.key_pressed(Key::F) && i.modifiers.is_none() {
            editor.frame_selection();
        }
    });
}
