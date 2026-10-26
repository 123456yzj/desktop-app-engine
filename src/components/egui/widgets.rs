//! Shared native component context and catalogue dispatch.
use super::*;
use crate::components::contracts::{PropertyType, PropertyType::*};
use crate::v02::widgets as contract;
use serde_json::{Value, json};

use crate::components::catalogue::{Definition, LayoutKind};
macro_rules! component {
    ($properties:expr, $events:expr) => {
        component!($properties, $events, LayoutKind::Column);
    };
    ($properties:expr, $events:expr, $layout:expr) => {
        component!($properties, $events, $layout; None, &[]);
    };
    ($properties:expr, $events:expr; $primary:expr, $secondary:expr) => {
        component!($properties, $events, LayoutKind::Column; $primary, $secondary);
    };
    ($properties:expr, $events:expr, $layout:expr; $primary:expr, $secondary:expr) => {
        pub(super) const DEFINITION: Definition = Definition {
            properties: $properties,
            events: $events,
            layout: $layout,
            primary: $primary,
            secondary: $secondary,
        };
    };
}
pub(in crate::components) mod affix;
mod alert;
mod anchor;
mod app;
mod auto_complete;
mod avatar;
pub(super) mod badge;
pub(in crate::components) mod badge_count;
pub(super) mod bar_chart;
mod border_beam;
mod breadcrumb;
pub(super) mod button;
mod calendar;
pub(super) mod card;
mod carousel;
mod cascader;
pub(super) mod checkbox;
mod collapse;
mod color_picker;
mod column;
mod config_provider;
mod content;
mod date_picker;
mod descriptions;
pub(super) mod divider;
mod drawer;
mod dropdown;
mod empty;
mod flex;
mod float_button;
mod footer;
mod form;
pub(super) mod form_item;
mod grid;
mod header;
mod icon;
mod image;
pub(super) mod input;
pub(super) mod input_number;
mod layout;
pub(super) mod line_chart;
mod list;
mod listy;
mod masonry;
mod mentions;
mod menu;
mod message;
pub(super) mod metric;
mod modal;
mod module;
mod notification;
mod pagination;
pub(super) mod panel;
pub(super) mod plugin;
mod popconfirm;
mod popover;
pub(super) mod progress;
mod qr_code;
pub(super) mod radio;
mod rate;
mod result;
mod row;
mod segmented;
pub(super) mod select;
mod shared;
mod sider;
mod skeleton;
pub(super) mod slider;
pub(super) mod space;
mod spin;
pub(super) mod splitter;
mod steps;
pub(super) mod switch;
pub(super) mod table;
mod tabs;
pub(super) mod text;
mod time_picker;
mod timeline;
mod tooltip;
mod tour;
mod transfer;
mod tree;
mod tree_select;
mod upload;
mod watermark;

pub(in crate::components) fn definition(kind: Kind) -> Definition {
    match kind {
        Kind::Anchor => anchor::DEFINITION,
        Kind::Breadcrumb => breadcrumb::DEFINITION,
        Kind::Collapse => collapse::DEFINITION,
        Kind::Dropdown => dropdown::DEFINITION,
        Kind::Menu => menu::DEFINITION,
        Kind::Pagination => pagination::DEFINITION,
        Kind::Segmented => segmented::DEFINITION,
        Kind::Steps => steps::DEFINITION,
        Kind::Tabs => tabs::DEFINITION,
        Kind::Drawer => drawer::DEFINITION,
        Kind::Message => message::DEFINITION,
        Kind::Modal => modal::DEFINITION,
        Kind::Notification => notification::DEFINITION,
        Kind::Popconfirm => popconfirm::DEFINITION,
        Kind::Popover => popover::DEFINITION,
        Kind::Tooltip => tooltip::DEFINITION,
        Kind::Tour => tour::DEFINITION,
        Kind::Alert => alert::DEFINITION,
        Kind::Avatar => avatar::DEFINITION,
        Kind::BadgeCount => badge_count::DEFINITION,
        Kind::BorderBeam => border_beam::DEFINITION,
        Kind::Carousel => carousel::DEFINITION,
        Kind::Descriptions => descriptions::DEFINITION,
        Kind::Empty => empty::DEFINITION,
        Kind::FloatButton => float_button::DEFINITION,
        Kind::Icon => icon::DEFINITION,
        Kind::Image => image::DEFINITION,
        Kind::List => list::DEFINITION,
        Kind::Listy => listy::DEFINITION,
        Kind::QrCode => qr_code::DEFINITION,
        Kind::Result => result::DEFINITION,
        Kind::Skeleton => skeleton::DEFINITION,
        Kind::Spin => spin::DEFINITION,
        Kind::Timeline => timeline::DEFINITION,
        Kind::Watermark => watermark::DEFINITION,
        Kind::AutoComplete => auto_complete::DEFINITION,
        Kind::Calendar => calendar::DEFINITION,
        Kind::Cascader => cascader::DEFINITION,
        Kind::ColorPicker => color_picker::DEFINITION,
        Kind::DatePicker => date_picker::DEFINITION,
        Kind::Mentions => mentions::DEFINITION,
        Kind::Rate => rate::DEFINITION,
        Kind::Select => select::DEFINITION,
        Kind::TimePicker => time_picker::DEFINITION,
        Kind::Transfer => transfer::DEFINITION,
        Kind::Tree => tree::DEFINITION,
        Kind::TreeSelect => tree_select::DEFINITION,
        Kind::Upload => upload::DEFINITION,
        Kind::Checkbox => checkbox::DEFINITION,
        Kind::Switch => switch::DEFINITION,
        Kind::Radio => radio::DEFINITION,
        Kind::InputNumber => input_number::DEFINITION,
        Kind::Slider => slider::DEFINITION,
        Kind::FormItem => form_item::DEFINITION,
        Kind::Affix => affix::DEFINITION,
        Kind::App => app::DEFINITION,
        Kind::Badge => badge::DEFINITION,
        Kind::Button => button::DEFINITION,
        Kind::Card => card::DEFINITION,
        Kind::Column => column::DEFINITION,
        Kind::ConfigProvider => config_provider::DEFINITION,
        Kind::Content => content::DEFINITION,
        Kind::Divider => divider::DEFINITION,
        Kind::Flex => flex::DEFINITION,
        Kind::Footer => footer::DEFINITION,
        Kind::Form => form::DEFINITION,
        Kind::Grid => grid::DEFINITION,
        Kind::Header => header::DEFINITION,
        Kind::Input => input::DEFINITION,
        Kind::Layout => layout::DEFINITION,
        Kind::Masonry => masonry::DEFINITION,
        Kind::Metric => metric::DEFINITION,
        Kind::Module => module::DEFINITION,
        Kind::Panel => panel::DEFINITION,
        Kind::Plugin => plugin::DEFINITION,
        Kind::Progress => progress::DEFINITION,
        Kind::Row => row::DEFINITION,
        Kind::Sider => sider::DEFINITION,
        Kind::Space => space::DEFINITION,
        Kind::Splitter => splitter::DEFINITION,
        Kind::Table => table::DEFINITION,
        Kind::Text => text::DEFINITION,
        Kind::LineChart => line_chart::DEFINITION,
        Kind::BarChart => bar_chart::DEFINITION,
    }
}

struct WidgetContext<'a, 'events> {
    ui: &'a mut egui::Ui,
    n: &'a v02::Node,
    r: &'a Resolved,
    rect: egui::Rect,
    state: &'a crate::binding::State,
    memory: &'a mut Memory,
    value: &'a mut Value,
    collector: &'a mut Collector<'events>,
}

#[derive(Default)]
pub(super) struct Memory {
    pub properties: BTreeMap<String, Value>,
    pub search: String,
    pub select_active: Option<String>,
    pub checks: HashSet<String>,
    pub started: Option<f64>,
    pub dismissed: bool,
    pub was_open: bool,
    pub body: Option<egui::Rect>,
    pub layer: Option<egui::LayerId>,
}

pub(super) fn local(
    n: &v02::Node,
    state: &crate::binding::State,
    memory: &Memory,
    key: &str,
    default: Value,
) -> Value {
    if let Some(binding) = n.api.get(key)
        && matches!(binding, Binding::State { .. })
    {
        return binding.resolve(state).cloned().unwrap_or(default);
    }
    memory
        .properties
        .get(key)
        .cloned()
        .or_else(|| contract::property(n, state, key).cloned())
        .unwrap_or(default)
}
fn emit(
    ui: &egui::Ui,
    n: &v02::Node,
    memory: &mut Memory,
    collector: &mut Collector<'_>,
    event: &str,
    payload: Value,
) {
    if let Some(key) = contract::event_property(n.kind, event) {
        memory.properties.insert(key.into(), payload.clone());
    }
    collector.push(ui, event, None, payload);
}
fn item_key(item: &Value) -> &str {
    item.get("key")
        .or_else(|| item.get("value"))
        .and_then(Value::as_str)
        .unwrap_or("")
}
fn options<'a>(n: &'a v02::Node, state: &'a crate::binding::State) -> &'a [Value] {
    n.options
        .as_ref()
        .and_then(|b| b.resolve(state).ok())
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}
fn option_parts(option: &Value) -> (&str, &str, bool) {
    if let Some(s) = option.as_str() {
        (s, s, false)
    } else {
        (item_key(option), item_label(option), item_disabled(option))
    }
}
fn item_label(item: &Value) -> &str {
    item.get("label")
        .or_else(|| item.get("title"))
        .and_then(Value::as_str)
        .unwrap_or_else(|| item_key(item))
}
fn item_disabled(item: &Value) -> bool {
    item["disabled"].as_bool().unwrap_or(false)
}
fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}
fn toggle(values: &mut Vec<String>, key: &str) {
    if let Some(index) = values.iter().position(|x| x == key) {
        values.remove(index);
    } else {
        values.push(key.into());
    }
}

pub(super) fn prepare(
    tree: &crate::component::Tree,
    state: &crate::binding::State,
    resolved: &mut [Resolved],
    values: &BTreeMap<String, Value>,
    memories: &BTreeMap<String, Memory>,
) {
    for (id, node) in tree.nodes.iter().enumerate() {
        let n = &node.spec;
        resolved[id]
            .api
            .insert("@childCount".into(), json!(node.children.len()));
        if contract::is_widget(n.kind)
            && !matches!(n.value, Some(Binding::State { .. }))
            && let Some(value) = values.get(&n.id)
        {
            resolved[id].value = Some(value.clone());
        }
        if let Some(parent) = node.parent {
            let p = &tree.nodes[parent].spec;
            resolved[id].visible &= resolved[parent].visible;
            resolved[id].enabled &= resolved[parent].enabled;
            for key in ["@theme", "@size", "@locale", "@direction"] {
                if let Some(value) = resolved[parent].api.get(key).cloned() {
                    resolved[id].api.insert(key.into(), value);
                }
            }
            if matches!(p.kind, Kind::App | Kind::ConfigProvider) {
                resolved[id].enabled &= !contract::flag(p, state, "componentDisabled", false);
            }
            if p.kind == Kind::Spin && contract::flag(p, state, "spinning", true) {
                resolved[id].enabled = false;
            }
            if matches!(p.kind, Kind::Card | Kind::Panel)
                && contract::flag(p, state, "loading", false)
            {
                resolved[id].visible = false;
            }
            if matches!(p.kind, Kind::Tabs | Kind::Carousel | Kind::Collapse) {
                let selection = resolved[parent].value.as_ref().unwrap_or(&Value::Null);
                let child_index = tree.nodes[parent]
                    .children
                    .iter()
                    .position(|&child| child == id)
                    .unwrap_or(0);
                let active = match p.kind {
                    Kind::Tabs => {
                        let selected = selection.as_str().filter(|s| !s.is_empty()).or_else(|| {
                            contract::items(p, state)
                                .iter()
                                .find(|item| !item_disabled(item))
                                .map(item_key)
                        });
                        selected == Some(n.id.as_str())
                    }
                    Kind::Carousel => selection.as_u64().unwrap_or(0) as usize == child_index,
                    _ => selection
                        .as_array()
                        .is_some_and(|a| a.iter().any(|key| key.as_str() == Some(n.id.as_str()))),
                };
                resolved[id].visible &= active;
            }
            if matches!(p.kind, Kind::Modal | Kind::Drawer) {
                let open = memories
                    .get(&p.id)
                    .map(|m| local(p, state, m, "open", json!(false)))
                    .unwrap_or_else(|| {
                        contract::property(p, state, "open")
                            .cloned()
                            .unwrap_or(json!(false))
                    });
                resolved[id].visible &= open == json!(true);
            }
        }
        match n.kind {
            Kind::App => app::prepare(&mut resolved[id]),
            Kind::ConfigProvider => config_provider::prepare(&mut resolved[id]),
            _ => {}
        }
    }
    let modal = tree
        .nodes
        .iter()
        .enumerate()
        .filter(|(id, node)| {
            resolved[*id].visible && matches!(node.spec.kind, Kind::Modal | Kind::Drawer)
        })
        .filter(|(_, node)| {
            memories
                .get(&node.spec.id)
                .map(|m| local(&node.spec, state, m, "open", json!(false)))
                .unwrap_or_else(|| {
                    contract::property(&node.spec, state, "open")
                        .cloned()
                        .unwrap_or(json!(false))
                })
                == true
        })
        .map(|(id, _)| id)
        .next_back();
    if let Some(modal) = modal {
        for (id, r) in resolved.iter_mut().enumerate() {
            let mut current = Some(id);
            while let Some(node) = current {
                if node == modal {
                    break;
                }
                current = tree.nodes[node].parent;
            }
            if current.is_none() {
                r.enabled = false;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint(
    ui: &mut egui::Ui,
    n: &v02::Node,
    r: &Resolved,
    rect: egui::Rect,
    state: &crate::binding::State,
    memory: &mut Memory,
    value: &mut Value,
    collector: &mut Collector<'_>,
) {
    let before = value.clone();
    let event_start = collector.events.len();
    let mut builder = egui::UiBuilder::new()
        .id_salt((&n.id, "widget"))
        .max_rect(rect);
    if r.api.get("@direction").and_then(Value::as_str) == Some("rtl") {
        builder = builder.layout(egui::Layout::right_to_left(egui::Align::Center));
    }
    let mut child = ui.new_child(builder);
    child.set_clip_rect(rect.intersect(ui.clip_rect()));
    child.style_mut().override_font_id = Some(style_font(&n.style, ui.ctx()));
    if !r.enabled {
        child.disable();
    }
    let c = WidgetContext {
        ui: &mut child,
        n,
        r,
        rect,
        state,
        memory: &mut *memory,
        value: &mut *value,
        collector: &mut *collector,
    };
    match n.kind {
        Kind::Anchor => anchor::paint(c),
        Kind::Breadcrumb => breadcrumb::paint(c),
        Kind::Dropdown => dropdown::paint(c),
        Kind::Menu => menu::paint(c),
        Kind::Pagination => pagination::paint(c),
        Kind::Steps => steps::paint(c),
        Kind::Tabs => tabs::paint(c),
        Kind::Segmented => segmented::paint(c),
        Kind::Collapse => collapse::paint(c),
        Kind::AutoComplete => auto_complete::paint(c),
        Kind::Calendar => calendar::paint(c),
        Kind::Cascader => cascader::paint(c),
        Kind::ColorPicker => color_picker::paint(c),
        Kind::DatePicker => date_picker::paint(c),
        Kind::Mentions => mentions::paint(c),
        Kind::Rate => rate::paint(c),
        Kind::Select => select::paint(c),
        Kind::TimePicker => time_picker::paint(c),
        Kind::Transfer => transfer::paint(c),
        Kind::Tree => tree::paint(c),
        Kind::TreeSelect => tree_select::paint(c),
        Kind::Upload => upload::paint(c),
        Kind::Modal => modal::paint(c),
        Kind::Drawer => drawer::paint(c),
        Kind::Message => message::paint(c),
        Kind::Notification => notification::paint(c),
        Kind::Tooltip => tooltip::paint(c),
        Kind::Popover => popover::paint(c),
        Kind::Popconfirm => popconfirm::paint(c),
        Kind::Tour => tour::paint(c),
        Kind::Alert => alert::paint(c),
        Kind::Badge => badge::paint(c),
        Kind::Avatar => avatar::paint(c),
        Kind::BadgeCount => badge_count::paint(c),
        Kind::BorderBeam => border_beam::paint(c),
        Kind::Carousel => carousel::paint(c),
        Kind::Descriptions => descriptions::paint(c),
        Kind::Empty => empty::paint(c),
        Kind::FloatButton => float_button::paint(c),
        Kind::Icon => icon::paint(c),
        Kind::Image => image::paint(c),
        Kind::List => list::paint(c),
        Kind::Listy => listy::paint(c),
        Kind::QrCode => qr_code::paint(c),
        Kind::Result => result::paint(c),
        Kind::Skeleton => skeleton::paint(c),
        Kind::Spin => spin::paint(c),
        Kind::Timeline => timeline::paint(c),
        Kind::Watermark => watermark::paint(c),
        _ => {}
    }
    let changed = if before.is_number() && value.is_number() {
        before.as_f64() != value.as_f64()
    } else {
        *value != before
    };
    if changed && r.enabled {
        let pending = collector.events.split_off(event_start);
        collector.push(&child, "change", None, value.clone());
        for event in pending {
            collector.push(&child, &event.kind, event.target_key, event.payload);
        }
    } else if !changed {
        *value = before;
    }
}

pub(super) fn header_rect(
    n: &v02::Node,
    r: &Resolved,
    rect: egui::Rect,
    content: egui::Rect,
) -> egui::Rect {
    if n.kind == Kind::Tabs && r.api.get("hideTabBar") == Some(&Value::Bool(true)) {
        return egui::Rect::from_min_size(rect.min, Vec2::ZERO);
    }
    if !matches!(n.kind, Kind::Tabs | Kind::Collapse | Kind::Carousel) {
        return content;
    }
    let inner = rect.shrink(n.layout.padding + n.style.border_width);
    match r
        .api
        .get("tabPlacement")
        .and_then(Value::as_str)
        .unwrap_or("top")
    {
        "start" => egui::Rect::from_min_max(inner.min, Pos2::new(content.left(), inner.bottom())),
        "end" => egui::Rect::from_min_max(Pos2::new(content.right(), inner.top()), inner.max),
        "bottom" => egui::Rect::from_min_max(Pos2::new(inner.left(), content.bottom()), inner.max),
        _ => egui::Rect::from_min_max(inner.min, Pos2::new(inner.right(), content.top())),
    }
}
