//! Dark, compact editor theme (inspired by Noggit / Qt dark styles).

use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Margin, Stroke, TextStyle, Visuals};

pub const ACCENT: Color32 = Color32::from_rgb(70, 135, 230);
pub const ACCENT_LIGHT: Color32 = Color32::from_rgb(120, 175, 255);

/// Side panels and menus.
pub const PANEL: Color32 = Color32::from_rgb(34, 35, 38);
/// Tool strip, status bar, text fields.
pub const PANEL_DARK: Color32 = Color32::from_rgb(25, 26, 28);
/// Floating toolbar over the viewport.
pub const OVERLAY: Color32 = Color32::from_rgba_premultiplied(15, 17, 21, 238);
pub const BORDER: Color32 = Color32::from_rgb(52, 54, 59);
pub const TEXT: Color32 = Color32::from_gray(205);
pub const TEXT_WEAK: Color32 = Color32::from_gray(135);

pub fn apply(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();
    visuals.panel_fill = PANEL;
    visuals.window_fill = PANEL;
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.extreme_bg_color = PANEL_DARK;
    visuals.faint_bg_color = Color32::from_rgb(40, 41, 45);
    visuals.selection.bg_fill = ACCENT;
    visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.hyperlink_color = ACCENT_LIGHT;

    let w = &mut visuals.widgets;
    w.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    w.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    w.inactive.weak_bg_fill = Color32::from_rgb(46, 47, 52);
    w.inactive.bg_fill = Color32::from_rgb(56, 58, 64);
    w.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(72, 75, 82));
    w.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    w.hovered.weak_bg_fill = Color32::from_rgb(58, 60, 66);
    w.hovered.bg_fill = Color32::from_rgb(58, 60, 66);
    w.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(80, 84, 92));
    w.active.weak_bg_fill = ACCENT;
    w.active.bg_fill = ACCENT;
    for widget in [&mut w.noninteractive, &mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] {
        widget.corner_radius = CornerRadius::same(3);
    }
    ctx.set_visuals(visuals);

    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (TextStyle::Heading, FontId::new(14.0, FontFamily::Proportional)),
            (TextStyle::Body, FontId::new(12.5, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(12.5, FontFamily::Proportional)),
            (TextStyle::Small, FontId::new(10.5, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(11.5, FontFamily::Monospace)),
        ]
        .into();
        style.spacing.item_spacing = egui::vec2(6.0, 4.0);
        style.spacing.button_padding = egui::vec2(6.0, 2.0);
        style.spacing.interact_size.y = 18.0;
        style.spacing.menu_margin = Margin::same(6);
        style.spacing.scroll.bar_width = 6.0;
    });
}
