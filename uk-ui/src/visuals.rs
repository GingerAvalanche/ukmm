use egui::epaint::{color_hex::color_from_hex, RectShape, Shadow, Tessellator};
use egui::{
    style::{Margin, Selection, Spacing, WidgetVisuals, Widgets},
    Color32, FontFamily, LayerId, Mesh, Rect, Rounding, Stroke, Style, Ui, Visuals,
};
use once_cell::sync::Lazy;

macro_rules! from_hex {
    ($hex:expr) => {{
        let _arr = color_from_hex!($hex);
        Color32::from_rgb(_arr[0], _arr[1], _arr[2])
    }};
}

pub const GREEN: Color32 = from_hex!("#528f24");
pub const BLUE: Color32 = from_hex!("#38b6f1");
pub const RED: Color32 = from_hex!("#F52331");
pub const YELLOW: Color32 = from_hex!("#ffbc28");
pub const ORGANGE: Color32 = from_hex!("#ff953f");

#[inline(always)]
pub fn error_bg(visuals: &Visuals) -> Color32 {
    let mut color = egui::color::Hsva::from(RED);
    color.v = egui::color::Hsva::from(visuals.window_fill()).v;
    color.into()
}

pub fn slate_grid(ui: &mut Ui) {
    ui.with_layer_id(LayerId::background(), |ui| {
        let cursor = ui.cursor();
        let width = ui.available_width();
        let height = ui.available_height() * 1.5;
        static GRID_COLOR: Lazy<Color32> = Lazy::new(|| BLUE.linear_multiply(0.0333));
        const GRID_OFFSET: f32 = 16.0;
        let bg_rect = Rect::from_min_size(ui.cursor().min, ui.available_size()); //.shrink(4.0);
        ui.painter().rect_filled(
            bg_rect,
            Rounding::none(),
            ui.style().visuals.extreme_bg_color,
        );
        ui.set_clip_rect(bg_rect);
        ui.painter().add({
            let mut mesh = Mesh::default();
            let mut tesselator = Tessellator::new(
                ui.fonts().pixels_per_point(),
                egui::epaint::TessellationOptions {
                    feathering: true,
                    feathering_size_in_pixels: 32.0,
                    ..Default::default()
                },
                [0, 0],
                vec![],
            );
            tesselator.tessellate_rect(
                &RectShape::stroke(
                    bg_rect.expand2([64.0, 0.0].into()),
                    0.0,
                    Stroke::new(2.0, ui.style().visuals.widgets.inactive.bg_fill),
                ),
                &mut mesh,
            );
            mesh
        });
        for i in 0..(height as usize / 48 + 1) {
            ui.painter().hline(
                cursor.min.x..=width + 4.0,
                (i as f32 * 48.0) + cursor.min.y + GRID_OFFSET,
                Stroke::new(1.0, *GRID_COLOR),
            );
        }
        for i in 0..(width as usize / 48 + 1) {
            ui.painter().vline(
                (i as f32 * 48.0) + cursor.min.x + GRID_OFFSET,
                cursor.min.y..=height,
                Stroke::new(1.0, *GRID_COLOR),
            );
        }
    });
}

pub fn default_dark(ctx: &egui::Context) {
    ctx.set_style(Style {
        animation_time: 0.2,
        visuals: Visuals {
            dark_mode: true,
            override_text_color: None,
            widgets: Widgets {
                noninteractive: WidgetVisuals {
                    bg_fill: from_hex!("#1C1E1F"),
                    bg_stroke: Stroke::new(1.0, from_hex!("#2F2E2A")),
                    fg_stroke: Stroke::new(1.0, from_hex!("#BCCAD1")),
                    rounding: Rounding::same(0.0),
                    expansion: 0.0,
                },
                inactive: WidgetVisuals {
                    bg_fill: from_hex!("#1d4e77"),
                    bg_stroke: Stroke::new(1.0, from_hex!("#237ba3")),
                    fg_stroke: Stroke::new(1.0, from_hex!("#f0f0f0")),
                    rounding: Rounding::same(2.0),
                    expansion: 0.0,
                },
                hovered: WidgetVisuals {
                    bg_fill: from_hex!("#237ba3"),
                    bg_stroke: Stroke::new(1.0, from_hex!("#1d649a")),
                    fg_stroke: Stroke::new(1.5, from_hex!("#f0f0f0")),
                    rounding: Rounding::same(2.0),
                    expansion: 1.0,
                },
                active: WidgetVisuals {
                    bg_fill: from_hex!("#12384f"),
                    bg_stroke: Stroke::new(1.0, from_hex!("#237ba3")),
                    fg_stroke: Stroke::new(1.5, from_hex!("#D9EEFF")),
                    rounding: Rounding::same(2.0),
                    expansion: 1.0,
                },
                open: WidgetVisuals {
                    bg_fill: from_hex!("#1C1E1F"),
                    bg_stroke: Stroke::new(1.0, from_hex!("#2F2E2A")),
                    fg_stroke: Stroke::new(1.0, from_hex!("#D9EEFF")),
                    rounding: Rounding::same(2.0),
                    expansion: 0.0,
                },
            },
            selection: Selection {
                bg_fill: BLUE.linear_multiply(0.667),
                stroke: Stroke::new(1.0, Color32::WHITE),
            },
            hyperlink_color: BLUE,
            faint_bg_color: from_hex!("#252729"),
            extreme_bg_color: from_hex!("#030a0e"), // e.g. TextEdit background
            code_bg_color: Color32::from_gray(32),
            warn_fg_color: ORGANGE, // orange
            error_fg_color: RED,    // red
            window_rounding: Rounding::same(4.0),
            window_shadow: Shadow::big_dark(),
            popup_shadow: Shadow::small_dark(),
            resize_corner_size: 8.0,
            text_cursor_width: 2.0,
            text_cursor_preview: false,
            clip_rect_margin: 3.0, // should be at least half the size of the widest frame stroke + max WidgetVisuals::expansion
            button_frame: true,
            collapsing_header_frame: false,
        },
        spacing: Spacing {
            button_padding: [4.0, 2.0].into(),
            icon_spacing: 4.0,
            menu_margin: Margin::same(4.0),
            scroll_bar_width: 2.0,
            indent_ends_with_horizontal_line: false,
            ..Default::default()
        },
        text_styles: {
            let mut styles = egui::style::default_text_styles();
            styles.get_mut(&egui::TextStyle::Heading).unwrap().family =
                FontFamily::Name("Bold".into());
            styles
        },
        ..Default::default()
    })
}