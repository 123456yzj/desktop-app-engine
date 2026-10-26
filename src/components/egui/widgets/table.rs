use super::*;
component!(
    &[
        ("pageSize", Integer(1, 10000)),
        ("current", Integer(1, 1000000000)),
        ("loading", Bool),
        ("emptyText", Text),
        ("bordered", Bool)
    ],
    &["rowSelect", "sortChange", "pageChange"]; None, &[("pageChange", "current")]
);
#[allow(clippy::too_many_arguments)]
pub(in crate::components::egui) fn paint(
    ui: &mut egui::Ui,
    p: &egui::Painter,
    n: &protocol::Node,
    rect: egui::Rect,
    enabled: bool,
    state: &crate::binding::State,
    version: u64,
    t: &mut TableState,
    events: &mut Collector<'_>,
) {
    if contract::flag(n, state, "loading", false) {
        ui.put(rect, egui::Spinner::new());
        return;
    }
    let rows = n
        .rows
        .as_ref()
        .unwrap()
        .resolve(state)
        .unwrap()
        .as_array()
        .unwrap();
    let columns: Vec<_> = crate::table::columns(n).collect();
    let key = n.row_key.as_ref().unwrap();
    let h = n.row_height.unwrap_or(32.);
    let page_size = contract::property(n, state, "pageSize")
        .and_then(Value::as_u64)
        .map(|v| v as usize);
    let pages = page_size.map_or(1, |size| rows.len().div_ceil(size).max(1));
    let requested = contract::number(n, state, "current", 1.) as u64;
    let controlled = matches!(n.api.get("current"), Some(Binding::State { .. }));
    let page = if controlled {
        requested
    } else {
        t.page.unwrap_or(requested)
    }
    .clamp(1, pages as u64);
    let page_start = page_size.map_or(0, |size| (page as usize - 1) * size);
    let page_len = page_size.map_or(rows.len(), |size| {
        rows.len().saturating_sub(page_start).min(size)
    });
    if t.page != Some(page) {
        t.scroll_y = 0.;
        t.page = Some(page);
    }
    let footer = if page_size.is_some() { 34. } else { 0. };
    let original_rect = rect;
    let rect = egui::Rect::from_min_max(rect.min, rect.max - Vec2::new(0., footer));
    if contract::flag(n, state, "bordered", false) {
        p.rect_stroke(
            original_rect,
            0.,
            Stroke::new(1_f32, token(ui, Token::Border)),
            StrokeKind::Inside,
        );
    }
    let font = n.style.font_size.unwrap_or(14.);
    let text_color = token(ui, n.style.color.unwrap_or(Token::TextPrimary));
    if t.version != version || t.widths.len() != columns.len() {
        t.order = if let Some((field, descending)) = &t.sort {
            crate::table::sorted_indices(rows, field, *descending)
        } else {
            (0..rows.len()).collect()
        };
        t.widths =
            crate::table::column_widths(&columns, rows, rect.width() - 12., &t.overrides, |s| {
                p.layout_no_wrap(s.into(), FontId::proportional(font), Color32::WHITE)
                    .size()
                    .x
            });
        t.version = version;
    }
    let fixed = t
        .widths
        .iter()
        .enumerate()
        .filter(|(i, _)| {
            columns[*i].width.fr.is_none() || t.overrides.get(*i).is_some_and(Option::is_some)
        })
        .map(|(_, w)| w)
        .sum::<f32>();
    let fr = columns
        .iter()
        .enumerate()
        .filter(|(i, c)| c.width.fr.is_some() && t.overrides.get(*i).is_none_or(Option::is_none))
        .map(|(_, c)| c.width.fr.unwrap())
        .sum::<f32>();
    if fr > 0. {
        for (i, c) in columns.iter().enumerate() {
            if let Some(weight) = c.width.fr
                && t.overrides.get(i).is_none_or(Option::is_none)
            {
                t.widths[i] = ((rect.width() - 12. - fixed) * weight / fr).max(48.);
            }
        }
    }
    let total = t.widths.iter().sum::<f32>();
    let header = egui::Rect::from_min_size(rect.min, Vec2::new((rect.width() - 12.).max(0.), 32.));
    let body = egui::Rect::from_min_max(header.left_bottom(), rect.max - Vec2::splat(12.));
    t.scroll_y = crate::table::clamp_scroll(t.scroll_y, page_len, h, body.height());
    t.scroll_x = t.scroll_x.clamp(0., (total - body.width()).max(0.));
    let head_p = p.with_clip_rect(header.intersect(ui.clip_rect()));
    let body_p = p.with_clip_rect(body.intersect(ui.clip_rect()));
    head_p.rect_filled(header, 0., token(ui, Token::SurfaceRaised));
    let mut x = header.left() - t.scroll_x;
    let mut sort_change = None;
    for (i, c) in columns.iter().enumerate() {
        let cell =
            egui::Rect::from_min_size(Pos2::new(x, header.top()), Vec2::new(t.widths[i], 32.));
        let label_text = format!(
            "{}{}",
            c.title,
            if t.sort.as_ref().is_some_and(|(f, _)| f == &c.field) {
                if t.sort.as_ref().unwrap().1 {
                    " ▼"
                } else {
                    " ▲"
                }
            } else {
                ""
            }
        );
        label_line(
            ui,
            &head_p,
            cell.shrink2(Vec2::new(8., 6.)),
            &label_text,
            font,
            text_color,
        );
        let response = ui.interact(
            cell.intersect(header).intersect(ui.clip_rect()),
            egui::Id::new((&n.id, &c.id, "sort")),
            if enabled {
                egui::Sense::click()
            } else {
                egui::Sense::hover()
            },
        );
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, &label_text)
        });
        let edge = egui::Rect::from_center_size(
            Pos2::new(cell.right(), cell.center().y),
            Vec2::new(6., 32.),
        );
        let drag = ui.interact(
            edge.intersect(header).intersect(ui.clip_rect()),
            egui::Id::new((&n.id, &c.id, "resize")),
            if enabled {
                egui::Sense::drag()
            } else {
                egui::Sense::hover()
            },
        );
        if drag.dragged() {
            let w = (t.widths[i] + ui.input(|i| i.pointer.delta().x)).clamp(48., 4000.);
            t.overrides.resize(columns.len(), None);
            t.overrides[i] = Some(w);
            t.widths[i] = w;
            ui.ctx().request_repaint();
        }
        if response.clicked() && !drag.dragged() {
            let desc = t.sort.as_ref().is_some_and(|(f, d)| f == &c.field && !*d);
            sort_change = Some((c.field.clone(), desc, c.id.clone()));
        }
        head_p.line_segment(
            [cell.right_top(), cell.right_bottom()],
            Stroke::new(1_f32, token(ui, Token::Border)),
        );
        x += t.widths[i];
    }
    if let Some((field, desc, id)) = sort_change {
        t.sort = Some((field.clone(), desc));
        t.order = crate::table::sorted_indices(rows, &field, desc);
        events.push(
            ui,
            "sortChange",
            Some(id),
            json!({"field":field,"descending":desc}),
        );
    }
    let range = crate::table::visible_rows(page_len, h, t.scroll_y, body.height());
    t.painted_rows = 0;
    if rows.is_empty() {
        label(
            ui,
            &body_p,
            body.shrink(8.),
            contract::string(n, state, "emptyText", "暂无数据"),
            font,
            token(ui, Token::TextMuted),
        );
    }
    let mut keyboard = None;
    for index in range {
        let row = &rows[t.order[page_start + index]];
        let row_key = row[key].as_str().unwrap();
        let row_rect = egui::Rect::from_min_size(
            Pos2::new(body.left(), body.top() + index as f32 * h - t.scroll_y),
            Vec2::new(body.width(), h),
        );
        if !row_rect
            .intersect(body)
            .intersect(ui.clip_rect())
            .is_positive()
        {
            continue;
        }
        t.painted_rows += 1;
        let response = ui.interact(
            row_rect.intersect(body).intersect(ui.clip_rect()),
            egui::Id::new((&n.id, row_key)),
            if enabled {
                egui::Sense::click()
            } else {
                egui::Sense::hover()
            },
        );
        let selected = t.selected.as_deref() == Some(row_key);
        if selected || response.hovered() || index % 2 == 1 {
            body_p.rect_filled(
                row_rect,
                0.,
                if selected {
                    ui.visuals().selection.bg_fill.gamma_multiply(0.4)
                } else {
                    token(ui, Token::SurfaceRaised)
                },
            );
        }
        if response.has_focus() {
            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                keyboard = Some((index + 1).min(page_len - 1));
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                keyboard = Some(index.saturating_sub(1));
            }
        }
        if response.clicked() {
            response.request_focus();
            if t.select(row_key.to_owned()) {
                events.push(ui, "rowSelect", Some(row_key.into()), json!(row_key));
            }
        }
        let mut x = body.left() - t.scroll_x;
        let mut summary = String::new();
        for (i, c) in columns.iter().enumerate() {
            let cell =
                egui::Rect::from_min_size(Pos2::new(x, row_rect.top()), Vec2::new(t.widths[i], h));
            let text = c.format.apply(&row[&c.field]).unwrap();
            summary.push_str(&format!("{}: {}; ", c.title, text));
            let cp = body_p.with_clip_rect(cell.intersect(body_p.clip_rect()));
            label_line(
                ui,
                &cp,
                cell.shrink2(Vec2::new(8., 6.)),
                &text,
                font,
                text_color,
            );
            x += t.widths[i];
        }
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::SelectableLabel,
                enabled,
                selected,
                &summary,
            )
        });
    }
    if let Some(index) = keyboard {
        let row_key = rows[t.order[page_start + index]][key].as_str().unwrap();
        if t.select(row_key.into()) {
            events.push(ui, "rowSelect", Some(row_key.into()), json!(row_key));
        }
        let top = index as f32 * h;
        if top < t.scroll_y {
            t.scroll_y = top;
        } else if top + h > t.scroll_y + body.height() {
            t.scroll_y = top + h - body.height();
        }
        ui.memory_mut(|m| m.request_focus(egui::Id::new((&n.id, row_key))));
        ui.ctx().request_repaint();
    }
    scrollbar(
        ui,
        p,
        (&n.id, "table-y"),
        egui::Rect::from_min_size(body.right_top(), Vec2::new(12., body.height())),
        &mut t.scroll_y,
        page_len as f32 * h,
        body.height(),
        false,
        enabled,
    );
    scrollbar(
        ui,
        p,
        (&n.id, "table-x"),
        egui::Rect::from_min_size(body.left_bottom(), Vec2::new(body.width(), 12.)),
        &mut t.scroll_x,
        total,
        body.width(),
        true,
        enabled,
    );
    if page_size.is_some() {
        let mut footer_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt((&n.id, "pagination"))
                .max_rect(egui::Rect::from_min_max(
                    Pos2::new(original_rect.left(), rect.bottom()),
                    original_rect.max,
                )),
        );
        let mut next = page;
        footer_ui.horizontal(|ui| {
            if ui
                .add_enabled(enabled && page > 1, egui::Button::new("‹"))
                .clicked()
            {
                next -= 1;
            }
            ui.label(format!("{page} / {pages}"));
            if ui
                .add_enabled(enabled && page < pages as u64, egui::Button::new("›"))
                .clicked()
            {
                next += 1;
            }
        });
        if next != page {
            t.page = Some(next);
            t.scroll_y = 0.;
            events.push(&footer_ui, "pageChange", None, json!(next));
        }
    }
}
