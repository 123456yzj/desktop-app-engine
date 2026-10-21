use super::*;
use serde_json::json;

#[test]
fn table_and_chart_painters_render_validated_data_and_refresh_caches() {
    let source = json!({"schemaVersion":"0.2","app":{"name":"Paint test","window":{"width":1000,"height":800}},"state":{"rows":[{"id":"a","value":1},{"id":"b","value":2}],"points":[{"x":0,"y":1},{"x":1,"y":3}]},"actions":{},"root":{"type":"column","id":"root","children":[
        {"type":"table","id":"table","layout":{"height":{"fixed":180}},"rows":{"kind":"state","key":"rows"},"rowKey":"id","columns":[{"id":"value","field":"value","title":"Value","width":{"fr":1}}]},
        {"type":"lineChart","id":"line","layout":{"height":{"fixed":240}},"series":[{"id":"line-series","name":"Trend","data":{"kind":"state","key":"points"},"xField":"x","yField":"y","monotonicX":true}]},
        {"type":"barChart","id":"bar","layout":{"height":{"fixed":240}},"series":[{"id":"bar-series","name":"Bars","data":{"kind":"state","key":"points"},"xField":"x","yField":"y"}]}
    ]}});
    let mut app = DashboardApp::embedded(v02::parse(&source.to_string()).unwrap()).unwrap();
    let context = egui::Context::default();
    for value in [3, 7] {
        app.runtime
            .set("points", json!([{"x":0,"y":1},{"x":1,"y":value}]))
            .unwrap();
        let output = context.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(1000.0, 800.0),
                )),
                ..Default::default()
            },
            |ctx| app.draw(ctx),
        );
        assert!(!output.shapes.is_empty());
        assert_eq!(app.tables["table"].painted_rows, 2);
        for id in ["line", "bar"] {
            assert_eq!(app.charts[id].points[0][1].unwrap().y, value as f64);
            assert!(!app.charts[id].sampled[0].is_empty());
        }
    }
}
