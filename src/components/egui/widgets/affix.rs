use super::*;
component!(
    &[
        ("offsetTop", Number(0., 10000.)),
        ("offsetBottom", Number(0., 10000.))
    ],
    &[]
);
pub(in crate::components) fn position(
    mut rect: crate::layout::Rect,
    clip: crate::layout::Rect,
    r: &Resolved,
) -> crate::layout::Rect {
    if let Some(bottom) = r.api.get("offsetBottom").and_then(Value::as_f64) {
        rect.y = rect.y.min(clip.y + clip.h - bottom as f32 - rect.h);
    } else {
        rect.y = rect
            .y
            .max(clip.y + r.api.get("offsetTop").and_then(Value::as_f64).unwrap_or(0.) as f32);
    }
    rect
}
