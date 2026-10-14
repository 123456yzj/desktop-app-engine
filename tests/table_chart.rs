use desktop_app_engine::{
    chart::{self, Point, Range, Transform},
    table,
};
use serde_json::json;
#[test]
fn virtual_rows_end_shrink_and_bound() {
    assert_eq!(table::visible_rows(10_000, 32.0, 0.0, 320.0), 0..12);
    assert_eq!(
        table::visible_rows(10_000, 32.0, 999999.0, 320.0),
        9988..10_000
    );
    assert_eq!(table::visible_rows(4, 32.0, 999999.0, 320.0), 0..4);
    assert_eq!(table::visible_rows(0, 32.0, 0.0, 320.0), 0..0);
    for scroll in (0..320000).step_by(731) {
        assert!(table::visible_rows(10_000, 32.0, scroll as f32, 320.0).len() <= 15);
    }
}
#[test]
fn stable_sort_null_last_and_selected_identity() {
    let rows = vec![
        json!({"id":"a","v":2}),
        json!({"id":"b","v":1}),
        json!({"id":"c","v":2}),
        json!({"id":"d","v":null}),
    ];
    assert_eq!(table::sorted_indices(&rows, "v", false), vec![1, 0, 2, 3]);
    assert_eq!(table::sorted_indices(&rows, "v", true), vec![0, 2, 1, 3]);
    let mut state = table::TableState::default();
    assert!(state.select("c".into()));
    assert!(!state.select("c".into()));
    state.order = table::sorted_indices(&rows, "v", false);
    assert_eq!(state.selected.as_deref(), Some("c"));
}
#[test]
fn column_sampling_and_override_share_widths() {
    let d = desktop_app_engine::v02::parse(include_str!("../examples/dashboard.json")).unwrap();
    let node = &d.root.children[5].0.children[0].0;
    let columns: Vec<_> = table::columns(node).collect();
    let rows = d.state["servers"].as_array().unwrap();
    let widths = table::column_widths(&columns, rows, 800.0, &[], |s| {
        s.chars().count() as f32 * 8.0
    });
    assert!((widths.iter().sum::<f32>() - 800.0).abs() < 0.01);
    let changed = table::column_widths(&columns, rows, 800.0, &[Some(200.0)], |s| {
        s.chars().count() as f32 * 8.0
    });
    assert_eq!(changed[0], 200.0);
    assert!((changed.iter().sum::<f32>() - 800.0).abs() < 0.01);
}
#[test]
fn transforms_ticks_extremes_and_clipped_hit() {
    let t = Transform {
        x: Range::new(-20.0, 80.0),
        y: Range::new(-100.0, 100.0),
        left: 50.0,
        top: 20.0,
        width: 800.0,
        height: 400.0,
    };
    for p in [
        Point {
            x: -20.0,
            y: -100.0,
        },
        Point { x: 37.0, y: 24.0 },
        Point { x: 80.0, y: 100.0 },
    ] {
        let q = t.data(t.screen(p));
        assert!((p.x - q.x).abs() < 1e-10 && (p.y - q.y).abs() < 1e-10);
    }
    for range in [
        Range::new(1.0, 1.0),
        Range::new(-1e50, 1e50),
        Range::new(1e-20, 2e-20),
        Range::new(1.00000001, 1.00000002),
        Range::new(1e50, 1e50 + 1e40),
    ] {
        let ticks = chart::ticks(range, 800.0);
        assert!(ticks.len() <= 64);
        assert!(ticks.iter().all(|v| v.is_finite()));
        assert!(ticks.windows(2).all(|t| t[0] < t[1]));
        let labels: std::collections::HashSet<_> =
            ticks.iter().map(|&n| chart::tick_text(n)).collect();
        assert_eq!(labels.len(), ticks.len());
    }
    let points = vec![
        Some(Point { x: 0.0, y: 0.0 }),
        None,
        Some(Point { x: 40.0, y: 40.0 }),
    ];
    assert_eq!(
        chart::hit(&points, t, t.screen(Point { x: 0.0, y: 0.0 }), true),
        Some(0)
    );
    assert_eq!(chart::hit(&points, t, Point { x: 0.0, y: 0.0 }, true), None);
    assert_eq!(
        chart::hit(&points, t, t.screen(Point { x: 20.0, y: 20.0 }), true),
        None
    );
}
#[test]
fn sampling_preserves_spikes_order_and_gaps() {
    let mut points: Vec<_> = (0..100_000)
        .map(|i| {
            Some(Point {
                x: i as f64,
                y: 0.0,
            })
        })
        .collect();
    points[50_001].as_mut().unwrap().y = 1000.0;
    points[50_002].as_mut().unwrap().y = -500.0;
    points[70_000] = None;
    let start = std::time::Instant::now();
    let sampled = chart::downsample(&points, Range::new(0.0, 99999.0), 1000);
    let elapsed = start.elapsed();
    assert!(sampled.contains(&50_001) && sampled.contains(&50_002) && sampled.contains(&70_000));
    assert!(sampled.windows(2).all(|w| w[0] < w[1]));
    assert!(sampled.len() <= 4010);
    println!(
        "SAMPLE points={} output={} width=1000 elapsed_us={} profile={}",
        points.len(),
        sampled.len(),
        elapsed.as_micros(),
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );
}

#[test]
fn nullable_cells_empty_single_and_viewport_anchor() {
    let mut s: serde_json::Value =
        serde_json::from_str(include_str!("../examples/dashboard.json")).unwrap();
    s["state"]["servers"][0]["load"] = json!(null);
    s["state"]["trend"][2]["cpu"] = json!(null);
    assert!(desktop_app_engine::v02::parse(&s.to_string()).is_ok());
    s["state"]["servers"][0]
        .as_object_mut()
        .unwrap()
        .remove("load");
    assert!(
        desktop_app_engine::v02::parse(&s.to_string())
            .unwrap_err()
            .contains("类型不一致")
            || desktop_app_engine::v02::parse(&s.to_string())
                .unwrap_err()
                .contains("缺少字段")
    );
    assert!(chart::downsample(&[], Range::new(0.0, 1.0), 800).is_empty());
    assert_eq!(
        chart::downsample(&[Some(Point { x: 1.0, y: 2.0 })], Range::new(1.0, 1.0), 800),
        vec![0]
    );
    let range = chart::zoom_range(Range::new(0.0, 10.0), 4.0, 0.5);
    assert_eq!(range.min, 2.0);
    assert_eq!(range.max, 7.0);
    assert_eq!(chart::pan_range(range, 3.0).min, 5.0);
}
