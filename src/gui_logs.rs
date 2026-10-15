//! Shared host diagnostics, independent of business state and JSON output parsing.
use desktop_app_engine::logging;
use eframe::egui;
use std::sync::atomic::Ordering;

#[derive(Default)]
pub struct Viewer {
    selected: Option<String>,
    latest: Option<String>,
    previous_error: Option<String>,
    open_error: Option<String>,
}
fn status(task: &logging::Task) -> String {
    task.finished()
        .map(|r| {
            match r["status"].as_str().unwrap_or_default() {
                "succeeded" => "成功",
                "failed" => "失败",
                "timedOut" => "超时",
                "cancelled" => "已取消",
                _ => "已结束",
            }
            .to_owned()
        })
        .unwrap_or_else(|| "运行中".into())
}
impl Viewer {
    pub fn show(&mut self, ctx: &egui::Context, error: Option<&str>) {
        if self.previous_error.as_deref() != error {
            if let Some(error) = error {
                logging::event("ERROR", "runtime.error", serde_json::json!({"error":error}));
            }
            self.previous_error = error.map(str::to_owned);
        }
        let Some(session) = logging::global() else {
            egui::TopBottomPanel::bottom("engine_logs").show(ctx, |ui| {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    format!("日志不可用：{}", logging::init_error().unwrap_or_default()),
                );
            });
            return;
        };
        let tasks = session.tasks();
        let active = tasks.iter().filter(|t| t.finished().is_none()).count();
        let latest = tasks.last().map(|t| t.id.clone());
        if latest != self.latest {
            self.selected = latest.clone();
            self.latest = latest;
        }
        egui::TopBottomPanel::bottom("engine_logs").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.button("日志与任务").clicked() {
                    session.viewer_open.store(true, Ordering::Relaxed);
                }
                if ui.button("打开日志目录").clicked() {
                    self.open_error = logging::open_directory(
                        session.directory.parent().unwrap_or(&session.directory),
                    )
                    .err();
                }
                if active > 0 {
                    ui.spinner();
                    ui.label(format!("{active} 个任务执行中 · 日志实时写入"));
                } else {
                    ui.weak("日志已自动保存");
                }
                if let Some(error) = session.error().or_else(|| self.open_error.clone()) {
                    ui.colored_label(ui.visuals().error_fg_color, error);
                }
            });
        });
        let mut open = session.viewer_open.load(Ordering::Relaxed);
        egui::Window::new("日志与任务").id(egui::Id::new("engine_log_viewer")).open(&mut open).default_size([860.0,580.0]).vscroll(true).show(ctx,|ui| {
            ui.weak(format!("本次运行：{}",session.directory.display()));
            ui.label("历史日志保存在磁盘；窗口预览每条流最近 32 KiB，完整内容可在任务目录查看。");
            if tasks.is_empty() { ui.label("尚未执行命令。启动和异常记录见下方引擎运行记录。"); }
            else {
                egui::ComboBox::from_id_salt("engine_task_picker").width(ui.available_width()-20.0)
                    .selected_text(tasks.iter().find(|t|Some(&t.id)==self.selected.as_ref()).map(|t|format!("{} · {} · {}",t.label,status(t),t.id)).unwrap_or_default())
                    .show_ui(ui,|ui| {
                        for task in tasks.iter().rev() {
                            ui.selectable_value(&mut self.selected,Some(task.id.clone()),format!("{} · {} · {}",task.label,status(task),task.id));
                        }
                    });
                if let Some(task)=tasks.iter().find(|t|Some(&t.id)==self.selected.as_ref()) {
                    ui.horizontal(|ui| {
                        ui.strong(format!("状态：{}",status(task)));
                        if let Some(result)=task.finished() { ui.label(format!("退出码：{} · 耗时 {} ms",result["exitCode"],result["durationMs"])); }
                        if ui.button("打开任务目录").clicked() { self.open_error=logging::open_directory(&task.directory).err(); }
                    });
                    if let Some(result)=task.finished() && let Some(error)=result["error"].as_str().filter(|s|!s.is_empty()) { ui.colored_label(ui.visuals().error_fg_color,error); }
                    if task.finished().is_some_and(|r|r["outputComplete"]==false) { ui.colored_label(ui.visuals().warn_fg_color,"任务结束时未确认输出完整关闭；日志可能仍在写入，读取错误见引擎运行记录。"); }
                    for (stderr,title) in [(false,"标准输出 stdout"),(true,"诊断输出 stderr")] {
                        egui::CollapsingHeader::new(format!("{title} · {} 字节",task.byte_count(stderr))).id_salt((task.id.as_str(),stderr)).default_open(true).show(ui,|ui| {
                            egui::ScrollArea::both().id_salt(("stream",task.id.as_str(),stderr)).max_height(210.0).stick_to_bottom(true).show(ui,|ui| {
                                let text=task.preview(stderr);
                                ui.add(egui::Label::new(egui::RichText::new(if text.is_empty() {"（暂无输出）"} else {&text}).monospace().size(12.0)).selectable(true));
                            });
                        });
                    }
                }
            }
            ui.separator();
            egui::CollapsingHeader::new("引擎运行记录").show(ui,|ui| {
                egui::ScrollArea::both().id_salt("engine_events").max_height(240.0).stick_to_bottom(true).show(ui,|ui| {
                    ui.add(egui::Label::new(egui::RichText::new(session.events()).monospace().size(12.0)).selectable(true));
                });
            });
            if let Some(error)=self.open_error.as_ref().or(session.error().as_ref()) { ui.colored_label(ui.visuals().error_fg_color,error); }
        });
        session.viewer_open.store(open, Ordering::Relaxed);
        if active > 0 || open {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}
