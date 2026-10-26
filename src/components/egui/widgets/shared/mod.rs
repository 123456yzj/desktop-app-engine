use super::*;
pub(super) mod card;
pub(super) mod config;
pub(super) mod date;
pub(super) mod dialog;
pub(super) mod image;
pub(super) mod list;
pub(super) mod menu;
pub(super) mod popup;
pub(super) mod selection;
pub(super) mod suggestions;
pub(super) mod toast;
pub(super) mod tree_selection;

/// Navigation targets use a complete hit area, distinct from text selection.
pub(in crate::components::egui::widgets) fn navigation_item(
    ui: &mut egui::Ui,
    label: &str,
    selected: bool,
    enabled: bool,
    width: f32,
    height: f32,
) -> egui::Response {
    let dark = ui.visuals().dark_mode;
    let accent = token(ui, Token::Primary);
    let fill = if selected {
        if dark {
            Color32::from_rgb(35, 49, 77)
        } else {
            Color32::from_rgb(237, 242, 255)
        }
    } else {
        Color32::TRANSPARENT
    };
    ui.scope(|ui| {
        let visuals = &mut ui.style_mut().visuals;
        visuals.widgets.inactive.weak_bg_fill = fill;
        visuals.widgets.hovered.weak_bg_fill = if dark {
            Color32::from_rgb(43, 48, 60)
        } else {
            Color32::from_rgb(243, 245, 249)
        };
        visuals.widgets.active.weak_bg_fill = if dark {
            Color32::from_rgb(40, 53, 80)
        } else {
            Color32::from_rgb(228, 235, 252)
        };
        let text = egui::RichText::new(label).size(14.).color(if !enabled {
            token(ui, Token::TextMuted)
        } else {
            ui.visuals().override_text_color.unwrap_or_else(|| {
                if selected {
                    accent
                } else {
                    token(ui, Token::TextPrimary)
                }
            })
        });
        ui.add_enabled(
            enabled,
            egui::Button::new(text)
                .min_size(Vec2::new(width, height))
                .corner_radius(6.)
                .stroke(Stroke::NONE),
        )
    })
    .inner
}

pub(in crate::components::egui::widgets) fn status_color(status: &str) -> Token {
    match status {
        "success" => Token::Success,
        "warning" => Token::Warning,
        "error" => Token::Danger,
        "default" => Token::TextMuted,
        _ => Token::Accent,
    }
}

pub(in crate::components::egui::widgets) fn icon(name: &str) -> &str {
    match name {
        "success" | "check" | "CheckOutlined" => "✓",
        "error" | "close" | "CloseOutlined" => "×",
        "warning" | "WarningOutlined" => "⚠",
        "search" | "SearchOutlined" => "⌕",
        "plus" | "PlusOutlined" => "+",
        "minus" | "MinusOutlined" => "−",
        "setting" | "SettingOutlined" => "⚙",
        "delete" | "DeleteOutlined" => "×",
        "left" | "LeftOutlined" => "‹",
        "right" | "RightOutlined" => "›",
        "up" | "UpOutlined" => "⌃",
        "down" | "DownOutlined" => "⌄",
        "loading" | "LoadingOutlined" => "◌",
        _ => "ⓘ",
    }
}
