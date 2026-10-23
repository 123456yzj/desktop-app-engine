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
