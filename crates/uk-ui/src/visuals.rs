use std::sync::LazyLock;

use color_hex::color_from_hex;
use egui::{
    epaint::{Margin, RectShape, Shadow, Tessellator},
    style::{Selection, Spacing, TextCursorStyle, WidgetVisuals, Widgets},
    vec2, Color32, FontFamily, LayerId, Mesh, Rect, Rounding, Stroke, Style, Ui, Visuals,
};
use egui_aesthetix::Aesthetix;
use serde::{Deserialize, Serialize};

macro_rules! hex_color {
    ($hex:expr) => {{
        let _arr = color_from_hex!($hex);
        Color32::from_rgb(_arr[0], _arr[1], _arr[2])
    }};
}

pub const GREEN: Color32 = hex_color!("#528f24");
pub const BLUE: Color32 = hex_color!("#38b6f1");
pub const RED: Color32 = hex_color!("#F52331");
pub const YELLOW: Color32 = hex_color!("#ffbc28");
pub const ORGANGE: Color32 = hex_color!("#ff953f");

#[inline(always)]
pub fn error_bg(visuals: &Visuals) -> Color32 {
    let mut color = egui::ecolor::Hsva::from(RED);
    color.v = egui::ecolor::Hsva::from(visuals.window_fill()).v;
    color.into()
}

pub fn style_dock(style: &egui::Style) -> egui_dock::Style {
    let mut dock_style = egui_dock::Style::from_egui(style);
    dock_style.tab.tab_body.rounding = Rounding {
        ne: 2.0,
        nw: 2.0,
        ..Default::default()
    };
    dock_style.tab.focused.text_color = style.visuals.strong_text_color();
    dock_style.tab.inactive.text_color = style.visuals.weak_text_color();
    dock_style.tab.tab_body.stroke.color = style.visuals.widgets.noninteractive.bg_stroke.color;
    dock_style.tab.active.outline_color = style.visuals.widgets.noninteractive.bg_stroke.color;
    dock_style.separator.width = 0.5;
    dock_style.separator.color_idle = style.visuals.widgets.noninteractive.bg_stroke.color;
    dock_style.separator.color_dragged = style.visuals.widgets.active.bg_stroke.color;
    dock_style.separator.color_hovered = style.visuals.widgets.active.bg_stroke.color;
    dock_style.dock_area_padding = Some(Margin::default());
    dock_style
}

pub fn slate_grid(ui: &mut Ui) {
    ui.with_layer_id(LayerId::background(), |ui| {
        let cursor = ui.cursor();
        let width = ui.available_width();
        let height = ui.available_height() * 1.5;
        static GRID_COLOR: LazyLock<Color32> = LazyLock::new(|| BLUE.linear_multiply(0.0333));
        const GRID_OFFSET: f32 = 16.0;
        let bg_rect = Rect::from_min_size(ui.cursor().min, ui.available_size()); //.shrink(4.0);
        ui.painter()
            .rect_filled(bg_rect, Rounding::ZERO, ui.style().visuals.extreme_bg_color);
        ui.set_clip_rect(bg_rect);
        ui.painter().add({
            let mut mesh = Mesh::default();
            let mut tesselator = Tessellator::new(
                ui.fonts(|f| f.pixels_per_point()),
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum Theme {
    #[default]
    Sheikah,
    Egui,
    EguiLight,
    Frappe,
    Latte,
    Macchiato,
    Mocha,
    AdwaitaDark,
    AdwaitaLight,
    Carl,
    SweetDark,
    ALamentforTimelessness,
}

impl Theme {
    #[inline]
    pub fn name(&self) -> &str {
        match self {
            Theme::Sheikah => "Sheikah Slate",
            Theme::Egui => "egui Dark",
            Theme::EguiLight => "egui Light",
            Theme::Frappe => "Frappe",
            Theme::Latte => "Latte",
            Theme::Macchiato => "Macchiato",
            Theme::Mocha => "Mocha",
            Theme::AdwaitaDark => "Adwaita Dark",
            Theme::AdwaitaLight => "Adwaita Light",
            Theme::Carl => "Carl",
            Theme::SweetDark => "Sweet Dark",
            Theme::ALamentforTimelessness => "ALamentforTimelessness",
        }
    }

    #[inline]
    pub fn iter() -> impl Iterator<Item = Self> {
        [
            Theme::Sheikah,
            Theme::Egui,
            Theme::EguiLight,
            Theme::Frappe,
            Theme::Latte,
            Theme::Macchiato,
            Theme::Mocha,
            Theme::AdwaitaDark,
            Theme::AdwaitaLight,
            Theme::Carl,
            Theme::SweetDark,
            Theme::ALamentforTimelessness,
        ]
        .into_iter()
    }

    pub fn set_theme(&self, ctx: &egui::Context) {
        match self {
            Self::Sheikah => {
                ctx.set_style(Style {
                    animation_time: 0.2,
                    visuals: Visuals {
                        dark_mode: true,
                        override_text_color: None,
                        widgets: Widgets {
                            noninteractive: WidgetVisuals {
                                bg_fill: hex_color!("#1C1E1F"),
                                bg_stroke: Stroke::new(1.0, hex_color!("#2F2E2A")),
                                fg_stroke: Stroke::new(1.0, hex_color!("#BCCAD1")),
                                rounding: Rounding::same(0.0),
                                expansion: 0.0,
                                weak_bg_fill: Color32::TRANSPARENT,
                            },
                            inactive: WidgetVisuals {
                                bg_fill: hex_color!("#1d4e77"),
                                bg_stroke: Stroke::new(1.0, hex_color!("#237ba3")),
                                fg_stroke: Stroke::new(1.0, hex_color!("#f0f0f0")),
                                rounding: Rounding::same(2.0),
                                expansion: 0.0,
                                weak_bg_fill: Color32::TRANSPARENT,
                            },
                            hovered: WidgetVisuals {
                                bg_fill: hex_color!("#237ba3"),
                                bg_stroke: Stroke::new(1.0, hex_color!("#1d649a")),
                                fg_stroke: Stroke::new(1.5, hex_color!("#f0f0f0")),
                                rounding: Rounding::same(2.0),
                                expansion: 1.0,
                                weak_bg_fill: Color32::TRANSPARENT,
                            },
                            active: WidgetVisuals {
                                bg_fill: hex_color!("#12384f"),
                                bg_stroke: Stroke::new(1.0, hex_color!("#237ba3")),
                                fg_stroke: Stroke::new(1.5, hex_color!("#D9EEFF")),
                                rounding: Rounding::same(2.0),
                                expansion: 1.0,
                                weak_bg_fill: Color32::TRANSPARENT,
                            },
                            open: WidgetVisuals {
                                bg_fill: hex_color!("#1C1E1F"),
                                bg_stroke: Stroke::new(1.0, hex_color!("#2F2E2A")),
                                fg_stroke: Stroke::new(1.0, hex_color!("#D9EEFF")),
                                rounding: Rounding::same(2.0),
                                expansion: 0.0,
                                weak_bg_fill: Color32::TRANSPARENT,
                            },
                        },
                        selection: Selection {
                            bg_fill: BLUE.linear_multiply(0.667),
                            stroke:  Stroke::new(1.0, Color32::WHITE),
                        },
                        hyperlink_color: BLUE,
                        faint_bg_color: hex_color!("#252729"),
                        extreme_bg_color: hex_color!("#030a0e"), // e.g. TextEdit background
                        code_bg_color: Color32::from_gray(32),
                        warn_fg_color: ORGANGE, // orange
                        error_fg_color: RED,    // red
                        window_rounding: Rounding::same(4.0),
                        window_shadow: Shadow {
                            offset: egui::Vec2::new(0., 0.),
                            blur:   5.,
                            spread: 5.,
                            color:  Color32::from_black_alpha(45),
                        },
                        popup_shadow: Shadow {
                            offset: egui::Vec2::new(0., 0.),
                            blur:   5.,
                            spread: 5.,
                            color:  Color32::from_black_alpha(45),
                        },
                        window_fill: hex_color!("#1C1E1F"),
                        window_stroke: Stroke::NONE,
                        panel_fill: hex_color!("#1C1E1F"),
                        resize_corner_size: 8.0,
                        text_cursor: TextCursorStyle {
                            preview: false,
                            ..Default::default()
                        },
                        clip_rect_margin: 3.0, /* should be at least half the size of the widest
                                                * frame stroke
                                                * + max WidgetVisuals::expansion */
                        button_frame: true,
                        collapsing_header_frame: false,
                        ..Default::default()
                    },
                    spacing: Spacing {
                        button_padding: [4.0, 2.0].into(),
                        icon_spacing: 4.0,
                        menu_margin: Margin::same(4.0),
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
                });
            }

            Self::ALamentforTimelessness => {
                ctx.set_style(Style {
                    animation_time: 0.2,
                    visuals: Visuals {
                        dark_mode: true,
                        override_text_color: None,
                        widgets: Widgets {
                            noninteractive: WidgetVisuals {
                                // 非交互式控件的背景填充颜色 - 使用深邃的暗紫色
                                bg_fill: hex_color!("#1A1B26"), 
                                // 非交互式控件的背景边框颜色 - 使用稍亮的紫色
                                bg_stroke: Stroke::new(1.0, hex_color!("#353846")), 
                                // 非交互式控件的前景描边颜色（例如文本颜色）- 使用柔和的灰蓝色
                                fg_stroke: Stroke::new(1.0, hex_color!("#A9B1D6")), 
                                // 圆角
                                rounding: Rounding::same(2.0), 
                                // 扩展
                                expansion: 0.0,
                                // 非交互式控件的弱背景填充颜色 - 使用更深的暗紫色
                                weak_bg_fill: hex_color!("#16161E"), 
                            },
                            inactive: WidgetVisuals {
                                // 非活动状态控件的背景填充颜色 - 使用柔和的蓝紫色
                                bg_fill: hex_color!("#4C4F69"), 
                                // 非活动状态控件的背景边框颜色 - 使用稍暗的蓝紫色
                                bg_stroke: Stroke::new(1.0, hex_color!("#45475A")), 
                                // 非活动状态控件的前景描边颜色（例如文本颜色）- 使用明亮的灰白色
                                fg_stroke: Stroke::new(1.0, hex_color!("#CDD6F4")), 
                                // 圆角
                                rounding: Rounding::same(2.0), 
                                // 扩展
                                expansion: 0.0,
                                // 非活动状态控件的弱背景填充颜色 - 使用较深的蓝紫色
                                weak_bg_fill: hex_color!("#313244"), 
                            },
                            hovered: WidgetVisuals {
                                // 悬停状态控件的背景填充颜色 - 使用更亮的蓝紫色
                                bg_fill: hex_color!("#5C5F77"), 
                                // 悬停状态控件的背景边框颜色 - 使用带有绿色调的边框色
                                bg_stroke: Stroke::new(1.0, hex_color!("#64748B")), 
                                // 悬停状态控件的前景描边颜色（例如文本颜色）- 使用更亮的灰白色，略带蓝色调
                                fg_stroke: Stroke::new(1.5, hex_color!("#D9E0EE")), 
                                // 圆角
                                rounding: Rounding::same(3.0), 
                                // 扩展
                                expansion: 1.0,
                                // 悬停状态控件的弱背景填充颜色 - 使用中等蓝紫色
                                weak_bg_fill: hex_color!("#4C4F69"),
                            },
                            active: WidgetVisuals {
                                // 活动状态控件的背景填充颜色 - 使用带有一点红色调的深紫色
                                bg_fill: hex_color!("#6D5875"), 
                                // 活动状态控件的背景边框颜色 - 使用带有蓝色调的边框色
                                bg_stroke: Stroke::new(1.0, hex_color!("#748494")), 
                                // 活动状态控件的前景描边颜色（例如文本颜色）- 使用非常明亮的米白色
                                fg_stroke: Stroke::new(1.5, hex_color!("#FAE3B0")),
                                // 圆角
                                rounding: Rounding::same(2.0), 
                                // 扩展
                                expansion: 1.0,
                                // 活动状态控件的弱背景填充颜色 - 使用接近活动背景的颜色
                                weak_bg_fill: hex_color!("#5C5F77"),
                            },
                            open: WidgetVisuals {
                                // 打开状态控件的背景填充颜色 - 使用一种柔和的绿色调
                                bg_fill: hex_color!("#455A64"), 
                                // 打开状态控件的背景边框颜色 - 使用深绿色调的边框色
                                bg_stroke: Stroke::new(1.0, hex_color!("#37474F")), 
                                // 打开状态控件的前景描边颜色（例如文本颜色）- 使用明亮的灰白色
                                fg_stroke: Stroke::new(1.0, hex_color!("#ECEFF1")), 
                                // 圆角
                                rounding: Rounding::same(2.0), 
                                // 扩展
                                expansion: 0.0,
                                // 打开状态控件的弱背景填充颜色 - 使用稍暗的绿色调
                                weak_bg_fill: hex_color!("#37474F"),
                            },
                        },
                        // 选中文本的背景颜色 - 使用带有橙色调的选区颜色
                        selection: Selection {
                            bg_fill: ORGANGE.linear_multiply(0.5),
                            stroke: Stroke::new(1.0, hex_color!("#F9E2AF")),
                        },
                        // 超链接颜色 - 使用明亮的青色
                        hyperlink_color: hex_color!("#89B4FA"),
                        // 微弱可见的背景颜色 - 使用深灰色
                        faint_bg_color: hex_color!("#242535"),
                        // 最暗的背景颜色（例如文本编辑器的背景）- 使用接近黑色的深紫色
                        extreme_bg_color: hex_color!("#16161E"),
                        // 代码块的背景颜色 - 使用深灰色，与faint_bg_color相似
                        code_bg_color: hex_color!("#242535"),
                        // 警告文字颜色 - 使用橙色
                        warn_fg_color: ORGANGE,
                        // 错误文字颜色 - 使用红色
                        error_fg_color: RED,
                        // 窗口圆角
                        window_rounding: Rounding::same(4.0),
                        // 窗口阴影 - 使用柔和的黑色阴影
                        window_shadow: Shadow {
                            offset: egui::Vec2::new(0., 0.),
                            blur:   5.,
                            spread: 5.,
                            color:  Color32::from_black_alpha(96),
                        },
                        // 弹出窗口阴影 - 使用更柔和的黑色阴影
                        popup_shadow: Shadow {
                            offset: egui::Vec2::new(0., 0.),
                            blur:   5.,
                            spread: 5.,
                            color:  Color32::from_black_alpha(48),
                        },
                        // 窗口填充颜色 - 使用深紫色
                        window_fill: hex_color!("#1A1B26"),
                        // 窗口边框颜色 - 无边框
                        window_stroke: Stroke::NONE,
                        // 面板填充颜色 - 使用与窗口填充颜色相同的深紫色
                        panel_fill: hex_color!("#1A1B26"),
                        // 调整大小的角落大小
                        resize_corner_size: 8.0,
                        // 文本光标样式
                        text_cursor: TextCursorStyle {
                            preview: false,
                            ..Default::default()
                        },
                        // 裁剪矩形边距
                        clip_rect_margin: 3.0,
                        // 按钮是否有边框
                        button_frame: true,
                        // 折叠标题是否有边框
                        collapsing_header_frame: false,
                        ..Default::default()
                    },
                    spacing: Spacing {
                        // 按钮内边距
                        button_padding: [4.0, 2.0].into(),
                        // 图标间距
                        icon_spacing: 4.0,
                        // 菜单边距
                        menu_margin: Margin::same(4.0),
                        // 缩进是否以水平线结束
                        indent_ends_with_horizontal_line: false,
                        ..Default::default()
                    },
                    text_styles: {
                        let mut styles = egui::style::default_text_styles();
                        // 设置标题字体为粗体
                        styles.get_mut(&egui::TextStyle::Heading).unwrap().family =
                            FontFamily::Name("Bold".into());
                        styles
                    },
                    ..Default::default()
                });
            }

            Self::Egui => {
                ctx.set_visuals(egui::style::Visuals::dark());
            }
            Self::EguiLight => {
                ctx.set_visuals(egui::style::Visuals::light());
            }
            Self::Frappe => {
                catppuccin_egui::set_theme(ctx, catppuccin_egui::FRAPPE);
            }
            Self::Latte => {
                catppuccin_egui::set_theme(ctx, catppuccin_egui::LATTE);
            }
            Self::Macchiato => {
                catppuccin_egui::set_theme(ctx, catppuccin_egui::MACCHIATO);
            }
            Self::Mocha => {
                catppuccin_egui::set_theme(ctx, catppuccin_egui::MOCHA);
            }
            Self::AdwaitaDark => {
                ctx.set_style(egui_aesthetix::themes::StandardDark.custom_style());
            }
            Self::AdwaitaLight => {
                ctx.set_style(egui_aesthetix::themes::StandardLight.custom_style());
            }
            Self::Carl => {
                ctx.set_style(egui_aesthetix::themes::CarlDark.custom_style());
            }
            Self::SweetDark => {
                ctx.set_style(Style {
                    visuals: Visuals {
                        dark_mode: true,
                        override_text_color: None,
                        widgets: Widgets {
                            noninteractive: WidgetVisuals {
                                weak_bg_fill: hex_color!("#181B28"),
                                bg_fill: hex_color!("#181B28"),
                                bg_stroke: Stroke::new(1.0, hex_color!("#2F3B51")), // separators, indentation lines
                                fg_stroke: Stroke::new(1.0, hex_color!("#EEEEEE")), // normal text color
                                rounding: Rounding::same(2.0),
                                expansion: 0.0,
                            },
                            inactive: WidgetVisuals {
                                weak_bg_fill: hex_color!("#1B1E2D"), // button background
                                bg_fill: hex_color!("#303651"),      // checkbox background
                                bg_stroke: Stroke {
                                    color: hex_color!("#12141e"),
                                    width: 1.0,
                                },
                                fg_stroke: Stroke::new(1.0, hex_color!("#fefefe")), // button text
                                rounding: Rounding::same(2.0),
                                expansion: 0.0,
                            },
                            hovered: WidgetVisuals {
                                weak_bg_fill: hex_color!("#262C45"),
                                bg_fill: hex_color!("#262C45"),
                                bg_stroke: Stroke::new(1.0, hex_color!("#71f79f")), // e.g. hover over window edge or button
                                fg_stroke: Stroke::new(1.5, Color32::from_gray(240)),
                                rounding: Rounding::same(3.0),
                                expansion: 0.5,
                            },
                            active: WidgetVisuals {
                                weak_bg_fill: hex_color!("#31363D"),
                                bg_fill: hex_color!("#31363D"),
                                bg_stroke: Stroke::new(1.0, Color32::WHITE),
                                fg_stroke: Stroke::new(2.0, Color32::WHITE),
                                rounding: Rounding::same(2.0),
                                expansion: 0.5,
                            },
                            open: WidgetVisuals {
                                weak_bg_fill: hex_color!("#262C45"),
                                bg_fill: hex_color!("#c74ded"),
                                bg_stroke: Stroke::new(1.0, Color32::from_gray(60)),
                                fg_stroke: Stroke::new(1.0, Color32::from_gray(210)),
                                rounding: Rounding::same(2.0),
                                expansion: 0.0,
                            },
                        },
                        selection: Selection {
                            bg_fill: hex_color!("#c74ded"),
                            stroke: Stroke {
                                color: Color32::WHITE,
                                width: 1.0,
                            },
                        },
                        hyperlink_color: hex_color!("#c74ded"),
                        faint_bg_color: hex_color!("#161925"), // visible, but barely so
                        extreme_bg_color: hex_color!("#181B21"), // e.g. TextEdit background
                        code_bg_color: hex_color!("#0C0E15"),
                        warn_fg_color: hex_color!("#ff6a00"), // orange
                        error_fg_color: hex_color!("#ed254e"), // red
                        window_rounding: Rounding::same(4.0),
                        window_shadow: Shadow {
                            offset: vec2(10.0, 20.0),
                            blur: 15.0,
                            spread: 0.0,
                            color: Color32::from_black_alpha(96),
                        },
                        window_fill: hex_color!("#181B28"),
                        window_stroke: Stroke::new(1.0, Color32::from_gray(60)),
                        window_highlight_topmost: true,
                        menu_rounding: Rounding::same(6.0),
                        panel_fill: hex_color!("#181B28"),
                        popup_shadow: Shadow {
                            offset: vec2(6.0, 10.0),
                            blur: 8.0,
                            spread: 0.0,
                            color: Color32::from_black_alpha(96),
                        },
                        resize_corner_size: 12.0,
                        text_cursor: TextCursorStyle {
                            preview: false,
                            stroke: Stroke::new(2.0, Color32::from_rgb(192, 222, 255)),
                            ..Default::default()
                        },
                        clip_rect_margin: 3.0, // should be at least half the size of the widest frame stroke + max WidgetVisuals::expansion
                        button_frame: true,
                        collapsing_header_frame: false,
                        indent_has_left_vline: true,
                        striped: false,
                        slider_trailing_fill: false,
                        handle_shape: egui::style::HandleShape::Circle,
                        interact_cursor: None,
                        image_loading_spinners: true,
                        numeric_color_space: egui::style::NumericColorSpace::GammaByte,
                    },
                    spacing: Spacing {
                        item_spacing: vec2(8.0, 4.0),
                        window_margin: Margin::same(8.0),
                        menu_margin: Margin::same(8.0),
                        button_padding: vec2(8.0, 4.0),
                        indent: 28.0, // match checkbox/radio-button with `button_padding.x + icon_width + icon_spacing`
                        interact_size: vec2(48.0, 20.0),
                        slider_width: 100.0,
                        slider_rail_height: 8.0,
                        combo_width: 100.0,
                        text_edit_width: 280.0,
                        icon_width: 16.0,
                        icon_width_inner: 10.0,
                        icon_spacing: 6.0,
                        tooltip_width: 600.0,
                        menu_width: 160.0,
                        menu_spacing: 4.0,
                        combo_height: 200.0,
                        scroll: Default::default(),
                        indent_ends_with_horizontal_line: false,
                        default_area_size: vec2(600.0, 400.0)
                    },
                    ..Default::default()
                });
            }
        }
    }
}
