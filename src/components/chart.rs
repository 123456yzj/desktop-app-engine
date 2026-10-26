//! f64 linear plots, finite ticks, original-data hit testing and spike preserving sampling.
use crate::{
    binding::State,
    protocol::{Axis, Kind, Node, required},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
#[derive(Debug, Clone, Copy)]
pub struct Range {
    pub min: f64,
    pub max: f64,
}
impl Range {
    pub fn new(min: f64, max: f64) -> Self {
        if min == max {
            let pad = (min.abs() * 0.01).max(1.0);
            Self {
                min: min - pad,
                max: max + pad,
            }
        } else {
            Self { min, max }
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub x: Range,
    pub y: Range,
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}
impl Transform {
    pub fn screen(&self, p: Point) -> Point {
        Point {
            x: self.left + (p.x - self.x.min) / (self.x.max - self.x.min) * self.width,
            y: self.top + self.height
                - (p.y - self.y.min) / (self.y.max - self.y.min) * self.height,
        }
    }
    pub fn data(&self, p: Point) -> Point {
        Point {
            x: self.x.min + (p.x - self.left) / self.width * (self.x.max - self.x.min),
            y: self.y.min
                + (self.top + self.height - p.y) / self.height * (self.y.max - self.y.min),
        }
    }
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.left
            && p.x <= self.left + self.width
            && p.y >= self.top
            && p.y <= self.top + self.height
    }
}
pub fn ticks(range: Range, pixels: f64) -> Vec<f64> {
    let span = range.max - range.min;
    if !span.is_finite() || span <= 0.0 || pixels <= 0.0 {
        return vec![];
    }
    let raw = span / (pixels / 80.0).clamp(2.0, 20.0);
    let base = 10.0_f64.powf(raw.log10().floor());
    let fraction = raw / base;
    let step = (if fraction <= 1.0 {
        1.0
    } else if fraction <= 2.0 {
        2.0
    } else if fraction <= 5.0 {
        5.0
    } else {
        10.0
    }) * base;
    if step <= 0.0 || !step.is_finite() {
        return vec![];
    }
    let start = (range.min / step).ceil() * step;
    (0..64)
        .map(|i| start + f64::from(i) * step)
        .take_while(|n| *n <= range.max)
        .filter(|n| n.is_finite())
        .map(|n| if n.abs() < step * 1e-10 { 0.0 } else { n })
        .collect()
}
pub fn tick_text(n: f64) -> String {
    if n != 0.0 && !(0.001..1e7).contains(&n.abs()) {
        format!("{n:e}")
    } else {
        n.to_string()
    }
}
pub fn series_points(
    series: &crate::protocol::Series,
    state: &State,
) -> Result<Vec<Option<Point>>, String> {
    series
        .data
        .resolve(state)?
        .as_array()
        .ok_or("series.data 必须为数组")?
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let x = r
                .get(&series.x_field)
                .ok_or_else(|| format!("data[{i}].{} 缺少字段", series.x_field))?
                .as_f64()
                .ok_or("X 必须为数字")?;
            let y = r
                .get(&series.y_field)
                .ok_or_else(|| format!("data[{i}].{} 缺少字段", series.y_field))?;
            if !x.is_finite() || x.abs() > 1e100 {
                return Err("X 必须有限且绝对值 <= 1e100".into());
            }
            if y.is_null() {
                return Ok(None);
            }
            let y = y.as_f64().ok_or("Y 必须为数字或 null")?;
            if !x.is_finite() || !y.is_finite() || x.abs() > 1e100 || y.abs() > 1e100 {
                return Err("坐标必须有限且绝对值 <= 1e100".into());
            }
            Ok(Some(Point { x, y }))
        })
        .collect()
}
pub fn validate(node: &Node, state: &State) -> Result<(), String> {
    if node
        .layout
        .height
        .as_ref()
        .and_then(|t| t.fixed)
        .is_none_or(|h| h < 120.0)
    {
        return Err("图表 layout.height 必须 fixed >= 120".into());
    }
    let series = required(&node.series, "series")?;
    let mut ids = std::collections::HashSet::new();
    if series.len() > 32 {
        return Err("series 最多 32 个".into());
    }
    for s in series {
        if s.id.is_empty()
            || s.name.is_empty()
            || s.x_field.is_empty()
            || s.y_field.is_empty()
            || !ids.insert(&s.id)
        {
            return Err("系列 ID 为空或重复".into());
        }
        let points = series_points(s, state)?;
        if s.monotonic_x
            && points
                .iter()
                .flatten()
                .map(|p| p.x)
                .collect::<Vec<_>>()
                .windows(2)
                .any(|p| p[0] > p[1])
        {
            return Err(format!("系列 {:?} 声明 monotonicX 但 X 非单调", s.id));
        }
        if !s.monotonic_x && points.len() > 20_000 {
            return Err("非单调系列最多 20000 点，请显式声明并保证 monotonicX".into());
        }
        if node.kind == Kind::BarChart && points.len() > 2000 {
            return Err("柱图每系列最多 2000 点".into());
        }
    }
    for axis in [&node.x_axis, &node.y_axis].into_iter().flatten() {
        for n in [axis.min, axis.max].into_iter().flatten() {
            if !n.is_finite() || n.abs() > 1e100 {
                return Err("轴范围必须有限且绝对值 <= 1e100".into());
            }
        }
        if axis.min.zip(axis.max).is_some_and(|(a, b)| a >= b) {
            return Err("轴范围必须 min < max".into());
        }
    }
    Ok(())
}
pub fn zoom_range(range: Range, anchor: f64, factor: f64) -> Range {
    let min = anchor + (range.min - anchor) * factor;
    let max = anchor + (range.max - anchor) * factor;
    let span = max - min;
    if min.is_finite()
        && max.is_finite()
        && min.abs() <= 1e100
        && max.abs() <= 1e100
        && span >= 1e-100
        && max > min
    {
        Range { min, max }
    } else {
        range
    }
}
pub fn pan_range(range: Range, delta: f64) -> Range {
    let min = range.min + delta;
    let max = range.max + delta;
    if min.is_finite() && max.is_finite() && min.abs() <= 1e100 && max.abs() <= 1e100 && min < max {
        Range { min, max }
    } else {
        range
    }
}
/// Minimum distinct X spacing, independent of the input ordering.
pub fn bar_spacing(points: &[Option<Point>]) -> f64 {
    let mut xs: Vec<_> = points.iter().flatten().map(|p| p.x).collect();
    xs.sort_by(f64::total_cmp);
    xs.windows(2)
        .map(|w| w[1] - w[0])
        .filter(|d| *d > 0.)
        .fold(f64::INFINITY, f64::min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_spacing_handles_unordered_duplicate_and_missing_points() {
        let points = [
            Some(Point { x: 5., y: 1. }),
            None,
            Some(Point { x: 0., y: 1. }),
            Some(Point { x: 2., y: 1. }),
            Some(Point { x: 2., y: 3. }),
        ];
        assert_eq!(bar_spacing(&points), 2.);
        assert!(bar_spacing(&[]).is_infinite());
        assert!(bar_spacing(&points[..1]).is_infinite());
    }
}

pub fn axis_range(
    points: impl Iterator<Item = f64>,
    axis: Option<&Axis>,
    include_zero: bool,
) -> Range {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for p in points {
        min = min.min(p);
        max = max.max(p);
    }
    if !min.is_finite() {
        min = 0.0;
        max = 1.0;
    }
    if include_zero {
        min = min.min(0.0);
        max = max.max(0.0);
    }
    let range = Range::new(min, max);
    let min = axis.and_then(|a| a.min).unwrap_or(range.min);
    let max = axis.and_then(|a| a.max).unwrap_or(range.max);
    if min >= max {
        if axis.and_then(|a| a.min).is_some() {
            Range::new(min, min + min.abs().max(1.0))
        } else {
            Range::new(max - max.abs().max(1.0), max)
        }
    } else {
        Range { min, max }
    }
}

/// Retain first/last/min/max in each screen column; nulls delimit independent runs.
pub fn downsample(points: &[Option<Point>], range: Range, pixels: usize) -> Vec<usize> {
    let mut out = Vec::new();
    let mut bucket: Option<i64> = None;
    let mut selected: Vec<usize> = Vec::new();
    fn flush(out: &mut Vec<usize>, selected: &mut Vec<usize>) {
        selected.sort_unstable();
        selected.dedup();
        out.append(selected);
    }
    for (i, p) in points.iter().enumerate() {
        let Some(p) = p else {
            flush(&mut out, &mut selected);
            out.push(i);
            bucket = None;
            continue;
        };
        let b = ((p.x - range.min) / (range.max - range.min) * pixels.max(1) as f64)
            .floor()
            .clamp(-1.0, pixels as f64 + 1.0) as i64;
        if bucket != Some(b) {
            flush(&mut out, &mut selected);
            selected = vec![i, i, i, i];
            bucket = Some(b);
        } else {
            selected[1] = i;
            if p.y < points[selected[2]].unwrap().y {
                selected[2] = i;
            }
            if p.y > points[selected[3]].unwrap().y {
                selected[3] = i;
            }
        }
    }
    flush(&mut out, &mut selected);
    out
}
pub fn hit(
    points: &[Option<Point>],
    t: Transform,
    pointer: Point,
    monotonic: bool,
) -> Option<usize> {
    if !t.contains(pointer) {
        return None;
    }
    let data = t.data(pointer);
    let radius = (t.x.max - t.x.min) * 6.0 / t.width;
    let (start, end) = if monotonic && points.iter().all(Option::is_some) {
        (
            points
                .partition_point(|p| p.unwrap().x < data.x - radius)
                .saturating_sub(1),
            (points.partition_point(|p| p.unwrap().x <= data.x + radius) + 1).min(points.len()),
        )
    } else {
        (0, points.len())
    };
    let mut best = None;
    let mut distance = 36.0;
    for i in start..end {
        let Some(p) = points[i] else { continue };
        let p = t.screen(p);
        let mut candidate = i;
        let mut d = (p.x - pointer.x).powi(2) + (p.y - pointer.y).powi(2);
        if i > 0
            && let Some(previous) = points[i - 1]
        {
            let a = t.screen(previous);
            let dx = p.x - a.x;
            let dy = p.y - a.y;
            let u = if dx * dx + dy * dy > 0.0 {
                ((pointer.x - a.x) * dx + (pointer.y - a.y) * dy) / (dx * dx + dy * dy)
            } else {
                0.0
            };
            let u = u.clamp(0.0, 1.0);
            if u < 0.5 {
                candidate = i - 1;
            }
            d = d.min((a.x + u * dx - pointer.x).powi(2) + (a.y + u * dy - pointer.y).powi(2));
        }
        if d <= distance {
            distance = d;
            best = Some(candidate);
        }
    }
    best
}
