use super::{Active, Controller, LogView, Repo, Status};
use desktop_app_engine::protocol::v02;
use serde_json::{Value, json};

fn literal(value: impl Into<Value>) -> Value {
    json!({"kind":"literal","value":value.into()})
}
fn bound(key: &str) -> Value {
    json!({"kind":"state","key":key})
}
pub fn node_mut<'a>(node: &'a mut Value, id: &str) -> Option<&'a mut Value> {
    if node["id"] == id {
        return Some(node);
    }
    for child in node
        .get_mut("children")
        .and_then(Value::as_array_mut)
        .into_iter()
        .flatten()
    {
        if let Some(n) = node_mut(child, id) {
            return Some(n);
        }
    }
    None
}
fn set(root: &mut Value, id: &str, key: &str, value: Value) {
    if let Some(node) = node_mut(root, id) {
        node[key] = value;
    }
}
fn text(id: &str, value: &str) -> Value {
    json!({"type":"text","id":id,"layout":{"height":{"fixed":18}},"style":{"fontSize":12},"text":literal(value)})
}
fn button(id: &str, label: &str, width: f32, enabled: bool) -> Value {
    json!({"type":"button","id":id,"layout":{"width":{"fixed":width},"height":{"fixed":28}},"style":{"fontSize":11,"paddingX":7,"paddingY":6,"textWrap":false},"enabled":literal(enabled),"text":literal(label)})
}
fn branch_cell(
    repo: &Repo,
    model: &Controller,
    state: &mut serde_json::Map<String, Value>,
    status: &str,
) -> Value {
    let id = &repo.project.id;
    let refreshing = matches!(&model.active, Some(Active::Branches(active)) if active == id);
    let key = format!("target_{id}");
    state.insert(
        key.clone(),
        Value::String(if repo.target.is_empty() {
            repo.project.branch.clone()
        } else {
            repo.target.clone()
        }),
    );
    let options = if refreshing {
        json!([{"label":"刷新中…","value":"__loading__","disabled":true}])
    } else if repo.refresh_error {
        let mut options = vec![
            json!({"label":"刷新失败，请点击 Fetch 重试","value":"__error__","disabled":true}),
        ];
        options.extend(
            repo.branches
                .iter()
                .map(|branch| json!({"label":branch,"value":branch})),
        );
        json!(options)
    } else if repo.branches.is_empty() {
        json!([{"label":"请点击 Fetch 获取分支","value":"__empty__","disabled":true}])
    } else {
        json!(repo.branches)
    };
    let mut children = vec![
        json!({"type":"select","id":format!("target:{id}"),"layout":{"width":{"fr":1},"height":{"fixed":30}},"style":{"fontSize":11},"enabled":literal(model.idle()||refreshing),"value":bound(&key),"options":literal(options),"placeholder":literal(if refreshing {"刷新中…"} else {"分支未确认"})}),
    ];
    if !status.is_empty() {
        children.push(json!({"type":"badge","id":format!("status:{id}"),"layout":{"width":{"fixed":76},"height":{"fixed":22}},"style":{"fontSize":11},"tone":if status.contains("失败"){"danger"}else if status.contains("成功"){"success"}else{"muted"},"text":literal(status)}));
    }
    json!({"type":"row","id":format!("branchCell:{id}"),"layout":{"gap":6,"align":"center"},"children":children})
}
fn apply_styles(node: &mut Value, dark: bool) {
    let dark = dark || node["id"] == "logCard";
    let kind = node["type"].as_str().unwrap_or("").to_owned();
    let mut base = json!({"color":if dark{"#acbdd5"}else{"#465268"}});
    if matches!(kind.as_str(), "button" | "input" | "select") {
        base = json!({"color":if dark{"#dce5f3"}else{"#465268"},"background":if dark{"#273447"}else{"#ffffff"},"borderColor":if dark{"#45536a"}else{"#dfe3ec"},"borderWidth":1,"radius":6,"fontSize":12,"paddingX":9,"paddingY":6,"textWrap":false,
          "hover":{"background":if dark{"#34455c"}else{"#f2f5ff"},"borderColor":if dark{"#607494"}else{"#a9b7e9"}},"pressed":{"background":if dark{"#1d2838"}else{"#e7edff"}},"disabled":{"background":if dark{"#253041"}else{"#f7f8fb"},"color":if dark{"#67768d"}else{"#a6afbf"},"borderColor":if dark{"#344258"}else{"#e8ebf1"}}});
    } else if kind == "card" {
        base = json!({"background":"#ffffff","borderColor":"#e8ebf1","borderWidth":1,"radius":10});
    }
    if kind == "button" {
        base["textAlign"] = json!("center");
    }
    let own = node
        .get("style")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    base.as_object_mut().unwrap().extend(own);
    node["style"] = base;
    if let Some(children) = node.get_mut("children").and_then(Value::as_array_mut) {
        for child in children {
            apply_styles(child, dark);
        }
    }
}

pub fn document(template: &Value, model: &Controller) -> Result<v02::Description, String> {
    v02::parse(&document_json(template, model)?.to_string())
}
pub fn document_json(template: &Value, model: &Controller) -> Result<Value, String> {
    let mut doc = template.clone();
    let root = &mut doc["root"];
    let idle = model.idle();
    let modal = model.settings_open || model.pending.is_some();
    for id in [
        "repoCard",
        "rootRepo",
        "logCard",
        "projects",
        "logScroll",
        "logOutput",
        "scan",
        "build",
        "logSource",
    ] {
        if node_mut(root, id).is_none() {
            return Err(format!("界面缺少组件 {id}"));
        }
    }
    set(root, "workspace", "enabled", literal(!modal));
    set(root, "rootRepo", "enabled", literal(!modal));
    set(
        root,
        "repoCount",
        "text",
        literal(model.repos.len().to_string()),
    );
    set(
        root,
        "snapshotLabel",
        "text",
        literal(if matches!(model.active, Some(Active::Scan)) {
            "● 扫描并刷新分支…"
        } else if model.repos.is_empty() {
            "● 尚未扫描"
        } else if !model.workspace_ready {
            "● 请重新扫描"
        } else {
            "● 已载入扫描快照"
        }),
    );
    set(root, "settings", "enabled", literal(idle && !modal));
    for id in ["scan", "all", "clear", "selectVisible", "retry"] {
        set(root, id, "enabled", literal(idle));
    }
    set(
        root,
        "build",
        "enabled",
        literal(idle && model.workspace_ready && !model.selected.is_empty()),
    );
    set(
        root,
        "build",
        "text",
        literal(
            if matches!(model.active, Some(Active::Prepare(_) | Active::Build(..))) {
                "正在构建…"
            } else {
                "批量构建"
            },
        ),
    );
    set(
        root,
        "scan",
        "text",
        literal(if matches!(model.active, Some(Active::Scan)) {
            "扫描中…"
        } else {
            "重新扫描"
        }),
    );
    let visible: Vec<_> = model.repos.iter().filter(|r| model.visible(r)).collect();
    set(
        root,
        "selectVisible",
        "text",
        literal(
            if !visible.is_empty()
                && visible
                    .iter()
                    .filter(|r| r.project.available())
                    .all(|r| model.selected.contains(&r.project.id))
            {
                "☑"
            } else {
                "□"
            },
        ),
    );
    set(
        root,
        "retry",
        "visible",
        literal(!model.retry_ids().is_empty()),
    );
    set(root, "error", "visible", literal(!model.error.is_empty()));
    let error = model.error.lines().take(2).collect::<Vec<_>>().join(" · ");
    set(root, "error", "text", literal(error));
    set(
        root,
        "autoScroll",
        "text",
        literal(if model.auto_scroll {
            "☑ 自动滚动"
        } else {
            "□ 自动滚动"
        }),
    );
    let mut rows = vec![];
    let mut dynamic_state = serde_json::Map::new();
    let root_children = if let Some(repo) = &model.root_repo {
        let id = &repo.project.id;
        let refreshing = matches!(&model.active,Some(Active::Branches(active)) if active == id);
        let changing = matches!(&model.active,Some(Active::Change(active)) if active == id);
        let status = if changing {
            "操作中…"
        } else if repo.project.branch.is_empty() {
            "分支未确认"
        } else {
            &repo.status
        };
        let mut selector = branch_cell(repo, model, &mut dynamic_state, status);
        selector["layout"]["width"] = json!({"fr":1});
        vec![
            json!({"type":"row","id":"rootControls","layout":{"height":{"fixed":30},"gap":6,"align":"center"},"children":[
                selector,
                button(&format!("fetch:{id}"),if refreshing {"刷新中"} else {"Fetch"},49.0,idle),
                button(&format!("pull:{id}"),"Pull",42.0,idle&&!repo.project.branch.is_empty())
            ]}),
        ]
    } else {
        vec![
            json!({"type":"row","id":"rootControls","layout":{"height":{"fixed":30},"gap":6,"align":"center"},"children":[
            text(
                "rootUnavailable",
                if !model.workspace_ready {
                    "扫描后显示根仓库分支"
                } else {
                    "根目录不是独立 Git 仓库"
                },
            ),
            button("fetch:@root","Fetch",49.0,false),button("pull:@root","Pull",42.0,false)]}),
        ]
    };
    let mut root_children = root_children;
    root_children.insert(0, json!({"type":"text","id":"rootName","layout":{"height":{"fixed":13}},"style":{"fontSize":10,"color":"#7c8595"},"text":literal("根仓库分支")}));
    set(root, "rootRepo", "children", json!(root_children));
    set(
        root,
        "rootUnavailable",
        "layout",
        json!({"width":{"fr":1},"height":{"fixed":30}}),
    );
    for repo in visible {
        let p = &repo.project;
        let id = &p.id;
        let refreshing = matches!(&model.active,Some(Active::Branches(active)) if active==id);
        let change = matches!(&model.active,Some(Active::Change(active)) if active==id);
        let status = if change {
            "操作中…"
        } else if !p.available() {
            if p.branch.is_empty() {
                "分支未确认"
            } else {
                "缺少构建脚本"
            }
        } else {
            &repo.status
        };
        let mut choose = button(
            &format!("choose:{id}"),
            if model.selected.contains(id) {
                "☑"
            } else {
                "□"
            },
            34.0,
            idle && p.available(),
        );
        choose["style"] = json!({"fontSize":14,"paddingX":8,"paddingY":5,"background":"#ffffff","borderColor":"#ffffff","borderWidth":0,"color":if model.selected.contains(id){"#4166eb"}else{"#a2adbf"}});
        let branch_cell = branch_cell(repo, model, &mut dynamic_state, status);
        let letter = id
            .strip_prefix("web-")
            .unwrap_or(id)
            .chars()
            .next()
            .unwrap_or('?')
            .to_ascii_uppercase()
            .to_string();
        let (project_bg, project_fg) = match letter.as_str() {
            "A" => ("#f1ecfb", "#9168c7"),
            "B" => ("#fff3e5", "#be843e"),
            "C" => ("#eaf5f4", "#389592"),
            _ => ("#edf2ff", "#5276cd"),
        };
        rows.push(json!({"type":"grid","id":format!("repo:{id}"),"columns":[{"fixed":34},{"fixed":130},{"fr":1},{"fixed":223}],"layout":{"height":{"fixed":50},"gap":6,"align":"center"},"style":{"background":if model.selected.contains(id){"#f3f6ff"}else{"#ffffff"}},"children":[
            choose,{"type":"row","id":format!("identity:{id}"),"layout":{"gap":7,"align":"center"},"children":[
                {"type":"text","id":format!("icon:{id}"),"layout":{"width":{"fixed":26},"height":{"fixed":26}},"style":{"background":project_bg,"color":project_fg,"radius":6,"fontSize":12,"fontWeight":"bold","textAlign":"center"},"text":literal(letter)},
                {"type":"text","id":format!("name:{id}"),"layout":{"width":{"fr":1},"height":{"fixed":18}},"style":{"fontSize":11,"fontWeight":"bold","textWrap":false,"color":"#202837"},"text":literal(id.as_str())}
            ]},
            branch_cell,
            {"type":"row","id":format!("actions:{id}"),"layout":{"gap":5,"align":"center"},"children":[
                button(&format!("fetch:{id}"),if refreshing {"刷新中"} else {"Fetch"},49.0,idle),
                button(&format!("pull:{id}"),"Pull",42.0,idle&&!p.branch.is_empty()),
                button(&format!("build:{id}"),"打包",42.0,idle&&model.workspace_ready&&p.available()),
                button(&format!("logs:{id}"),"查看日志",70.0,repo.latest.is_some())
            ]}
        ]}));
    }
    if rows.is_empty() {
        rows.push(text(
            "empty",
            if model.repos.is_empty() {
                "暂无业务仓库"
            } else {
                "没有匹配的项目"
            },
        ));
    }
    set(root, "projects", "children", json!(rows));
    let mut options = vec![json!({"label":"当前会话","value":"session"})];
    for (r, run) in model.runs.iter().enumerate().rev() {
        options.push(
            json!({"label":format!("{} · 本批全部日志",run.id),"value":LogView::Run(r).key()}),
        );
        for (j, job) in run.jobs.iter().enumerate() {
            options.push(json!({"label":format!("{}/P{} · {} · {}",run.id,j+1,if run.plan.all{"全部项目"}else{&job.ids[0]},job.status.text()),"value":LogView::Job(r,j).key()}));
        }
    }
    let metadata = match model.log_view {
        LogView::Session => String::new(),
        LogView::Run(r) => {
            let run = &model.runs[r];
            format!(
                "{} · {} · {:.1} 秒",
                run.id,
                run.status.text(),
                run.duration
                    .unwrap_or_else(|| run.started.elapsed().as_secs_f32())
            )
        }
        LogView::Job(r, j) => {
            let run = &model.runs[r];
            let job = &run.jobs[j];
            let scope = if run.plan.all {
                "全部项目共享进程".into()
            } else {
                format!(
                    "分支：{}",
                    run.plan
                        .projects
                        .iter()
                        .find(|p| p.id == job.ids[0])
                        .map(|p| p.branch.as_str())
                        .unwrap_or("—")
                )
            };
            let result = job.task.as_ref().and_then(|t| t.finished());
            format!(
                "{}/P{} · {}\npnpm {}\n{}{}",
                run.id,
                j + 1,
                job.status.text(),
                job.command,
                scope,
                result
                    .map(|r| format!(
                        " · 退出码 {} · {:.1} 秒",
                        r["exitCode"],
                        r["durationMs"].as_u64().unwrap_or(0) as f64 / 1000.0
                    ))
                    .unwrap_or_else(|| {
                        if job.status == Status::Skipped {
                            format!(
                                " · {}",
                                if run.jobs.iter().any(|j| j.task.is_some()) {
                                    "前序进程失败"
                                } else {
                                    "准备阶段失败"
                                }
                            )
                        } else if job.status == Status::Waiting {
                            " · 尚未启动".into()
                        } else if let Some(task) = &job.task {
                            format!(
                                " · {:.1} 秒",
                                desktop_app_engine::services::logging::now_ms()
                                    .saturating_sub(task.started_ms)
                                    as f64
                                    / 1000.0
                            )
                        } else {
                            String::new()
                        }
                    })
            )
        }
    };
    set(
        root,
        "logMetadata",
        "visible",
        literal(!metadata.is_empty()),
    );
    for id in ["all", "clear", "retry"] {
        set(
            root,
            id,
            "style",
            json!({"fontSize":11,"color":"#5d73aa","background":"#ffffff","borderColor":"#ffffff","borderWidth":0,"paddingX":5,"paddingY":5,"textWrap":false}),
        );
    }
    apply_styles(root, false);
    let state = doc["state"].as_object_mut().ok_or("state 必须是对象")?;
    state.extend(dynamic_state);
    for (key, value) in [
        ("sshTarget", model.connection.host.clone()),
        ("repository", model.connection.root.clone()),
        ("hostDraft", model.draft_host.clone()),
        ("rootDraft", model.draft_root.clone()),
        ("searchValue", model.search.clone()),
        ("sourceValue", model.log_view.key()),
        (
            "connectionText",
            format!("{}    {}", model.connection.host, model.connection.root),
        ),
        (
            "selectionText",
            format!("已选 {} / {}", model.selected.len(), model.repos.len()),
        ),
        ("statusText", model.note.clone()),
        ("logText", model.log_text()),
        ("logMeta", metadata),
    ] {
        state.insert(key.into(), Value::String(value));
    }
    state.insert("sourceOptions".into(), json!(options));
    Ok(doc)
}

pub fn modal(template: &Value, model: &Controller) -> Result<v02::Description, String> {
    let mut doc = document_json(template, model)?;
    let children = if model.settings_open {
        vec![
            text("hostLabel", "SSH 地址 / 配置别名"),
            json!({"type":"input","id":"hostInput","layout":{"height":{"fixed":34}},"value":bound("hostDraft")}),
            text("rootLabel", "Turbo 工作区根目录"),
            json!({"type":"input","id":"rootInput","layout":{"height":{"fixed":34}},"value":bound("rootDraft")}),
            json!({"type":"row","id":"settingsButtons","layout":{"height":{"fixed":34},"gap":10,"justify":"end"},"children":[button("cancelSettings","取消",70.0,model.connection.validate().is_ok()),button("saveSettings","保存并扫描",110.0,true)]}),
        ]
    } else if let Some(plan) = &model.pending {
        let mut rows = vec![];
        if let Some(root) = &plan.root {
            rows.push(text(
                "reviewRoot",
                &format!("Turbo 根仓库    {}", root.branch),
            ));
        }
        for (i, p) in plan.projects.iter().enumerate() {
            rows.push(text(
                &format!("review{i}"),
                &format!("{}    {}", p.id, p.branch),
            ));
        }
        rows.push(text("cleanCommand", "rm -rf -- ./.turbo"));
        if plan.all {
            rows.push(text("allCommand", "pnpm build:dev"));
        } else {
            for (i, p) in plan.projects.iter().enumerate() {
                rows.push(text(
                    &format!("command{i}"),
                    &format!("pnpm {}", p.script.as_deref().unwrap_or_default()),
                ));
            }
        }
        rows.push(json!({"type":"row","id":"reviewButtons","layout":{"height":{"fixed":34},"gap":10,"justify":"end"},"children":[button("cancelBuild","返回调整",90.0,true),button("confirmBuild","确认构建",100.0,true)]}));
        rows
    } else {
        vec![]
    };
    doc["root"] = json!({"type":"column","id":"dialog","layout":{"width":{"fr":1},"height":{"fr":1},"padding":12,"gap":12,"overflow":"scroll"},"children":children});
    apply_styles(&mut doc["root"], false);
    v02::parse(&doc.to_string())
}
