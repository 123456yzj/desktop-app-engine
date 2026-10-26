//! Fixed-height virtual table model. The GUI formats only the returned row interval.
use crate::{
    binding::{FormatKind, State},
    protocol::{Column, ColumnSpec, Node, required},
};
use serde_json::Value;
use std::{cmp::Ordering, collections::HashSet, ops::Range};

pub fn columns(node: &Node) -> impl Iterator<Item = &Column> {
    node.columns.iter().filter_map(|c| match c {
        ColumnSpec::Data(c) => Some(c),
        _ => None,
    })
}
pub fn validate(node: &Node, state: &State) -> Result<(), String> {
    let rows = required(&node.rows, "rows")?
        .resolve(state)?
        .as_array()
        .ok_or("rows 必须是数组")?;
    let key = required(&node.row_key, "rowKey")?;
    let h = node.row_height.unwrap_or(32.0);
    if !h.is_finite() || !(16.0..=512.0).contains(&h) {
        return Err("rowHeight 必须在 16..512".into());
    }
    if node.columns.is_empty() || node.columns.len() > 64 {
        return Err("columns 数量必须在 1..64".into());
    }
    let mut ids = HashSet::new();
    let mut keys = HashSet::new();
    for c in &node.columns {
        let ColumnSpec::Data(c) = c else {
            return Err("table.columns 必须包含 id/field/title/width".into());
        };
        if c.id.is_empty() || c.field.is_empty() || !ids.insert(&c.id) {
            return Err("列 ID 为空/重复或 field 为空".into());
        }
        c.width.validate()?;
        if c.format.precision > 12 {
            return Err("format.precision 必须在 0..12".into());
        }
    }
    let mut types = vec![None; node.columns.len()];
    for (i, row) in rows.iter().enumerate() {
        let object = row
            .as_object()
            .ok_or_else(|| format!("rows[{i}] 必须是对象"))?;
        let id = object
            .get(key)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("rows[{i}].{key}: rowKey 必须是字符串"))?;
        if !keys.insert(id) {
            return Err(format!("rows[{i}].{key}: 重复 rowKey {id:?}"));
        }
        for (j, c) in columns(node).enumerate() {
            let v = object
                .get(&c.field)
                .ok_or_else(|| format!("rows[{i}].{}: 缺少字段", c.field))?;
            c.format
                .apply(v)
                .map_err(|e| format!("rows[{i}].{}: {e}", c.field))?;
            let t = if v.is_null() {
                continue;
            } else if v.is_number() {
                0
            } else if v.is_string() {
                1
            } else if v.is_boolean() && c.format.kind == FormatKind::Text {
                2
            } else {
                return Err("单元格类型非法".into());
            };
            if types[j].is_some_and(|prior| prior != t) {
                return Err(format!("列 {:?} 混合类型", c.id));
            }
            types[j] = Some(t);
        }
    }
    Ok(())
}

pub fn clamp_scroll(scroll: f32, count: usize, row_height: f32, viewport: f32) -> f32 {
    scroll.clamp(0.0, (count as f32 * row_height - viewport).max(0.0))
}
pub fn visible_rows(count: usize, row_height: f32, scroll: f32, viewport: f32) -> Range<usize> {
    if count == 0 || row_height <= 0.0 || viewport <= 0.0 {
        return 0..0;
    }
    let scroll = clamp_scroll(scroll, count, row_height, viewport);
    let start = (scroll / row_height).floor() as usize;
    let end = ((scroll + viewport) / row_height).ceil() as usize;
    start.saturating_sub(2)..end.saturating_add(2).min(count)
}

pub fn sorted_indices(rows: &[Value], field: &str, descending: bool) -> Vec<usize> {
    let mut order: Vec<_> = (0..rows.len()).collect();
    order.sort_by(|&a, &b| {
        let a = &rows[a][field];
        let b = &rows[b][field];
        match (a.is_null(), b.is_null()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            _ => {
                let cmp = match (a, b) {
                    (Value::Number(a), Value::Number(b)) => a
                        .as_f64()
                        .partial_cmp(&b.as_f64())
                        .unwrap_or(Ordering::Equal),
                    (Value::String(a), Value::String(b)) => a.cmp(b),
                    (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
                    _ => Ordering::Equal,
                };
                if descending { cmp.reverse() } else { cmp }
            }
        }
    });
    order
}

pub fn column_widths(
    columns: &[&Column],
    rows: &[Value],
    available: f32,
    overrides: &[Option<f32>],
    measure: impl Fn(&str) -> f32,
) -> Vec<f32> {
    let mut widths = Vec::new();
    let mut weight = 0.0;
    for (i, c) in columns.iter().enumerate() {
        let w = overrides.get(i).copied().flatten().unwrap_or_else(|| {
            if let Some(w) = c.width.fixed {
                w
            } else if c.width.auto == Some(true) {
                rows.iter()
                    .take(200)
                    .filter_map(|r| c.format.apply(&r[&c.field]).ok())
                    .map(|s| measure(&s) + 16.0)
                    .fold(measure(&c.title) + 16.0, f32::max)
            } else {
                weight += c.width.fr.unwrap_or(1.0);
                48.0
            }
        });
        widths.push(w.max(48.0));
    }
    let free = (available - widths.iter().sum::<f32>()).max(0.0);
    for (i, c) in columns.iter().enumerate() {
        if overrides.get(i).is_none_or(Option::is_none) && c.width.fr.is_some() && weight > 0.0 {
            widths[i] += free * c.width.fr.unwrap_or(1.0) / weight;
        }
    }
    widths
}

#[derive(Default)]
pub struct TableState {
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub selected: Option<String>,
    pub sort: Option<(String, bool)>,
    pub order: Vec<usize>,
    pub widths: Vec<f32>,
    pub overrides: Vec<Option<f32>>,
    pub version: u64,
    pub painted_rows: usize,
    pub page: Option<u64>,
}
impl TableState {
    pub fn select(&mut self, key: String) -> bool {
        if self.selected.as_ref() == Some(&key) {
            false
        } else {
            self.selected = Some(key);
            true
        }
    }
}
