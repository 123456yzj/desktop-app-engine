//! HTTP scheduling and atomic response commits; transport belongs to services.
use super::{Runtime, State};

impl Runtime {
    pub fn request_http(&mut self, source: &str) -> Result<bool, String> {
        self.request_http_snapshot(source, &self.state.clone())
    }
    pub(super) fn request_http_snapshot(
        &mut self,
        id: &str,
        snapshot: &State,
    ) -> Result<bool, String> {
        let source = self
            .description
            .data_sources
            .get(id)
            .ok_or_else(|| format!("未定义 dataSource {id:?}"))?
            .clone();
        if self.http_jobs.contains(id) || !source.is_enabled(snapshot)? {
            return Ok(false);
        }
        let request = source.resolve_with_store(snapshot, self.credential_store.clone())?;
        let updated = self.state[&source.status_target]["updatedAt"]
            .as_u64()
            .unwrap_or(0);
        self.set(
            &source.status_target,
            crate::http::status("loading", true, 0, "", updated),
        )?;
        let tx = self.http_tx.clone();
        let name = id.to_owned();
        if let Err(e) = std::thread::Builder::new()
            .name("http".into())
            .spawn(move || {
                let _ = tx.send((name, crate::http::run(request)));
            })
        {
            self.set(
                &source.status_target,
                crate::http::status("failed", false, 0, "无法启动请求线程", updated),
            )?;
            return Err(e.to_string());
        }
        self.http_jobs.insert(id.to_owned());
        self.http_due.remove(id);
        Ok(true)
    }
    /// Constructing or checking a document never sends a request.
    pub fn poll_http(&mut self) {
        while let Ok((id, response)) = self.http_rx.try_recv() {
            self.http_jobs.remove(&id);
            let source = self.description.data_sources[&id].clone();
            let old_updated = self.state[&source.status_target]["updatedAt"]
                .as_u64()
                .unwrap_or(0);
            let result = (|| {
                let body = response.result?;
                let mut updates = source.map(&body)?;
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| e.to_string())?
                    .as_millis() as u64;
                updates.insert(
                    source.status_target.clone(),
                    crate::http::status("success", false, response.code, "", now),
                );
                self.set_many(updates)
            })();
            if let Err(error) = result
                && let Err(e) = self.set(
                    &source.status_target,
                    crate::http::status("failed", false, response.code, &error, old_updated),
                )
            {
                self.error = Some(e);
            }
            if let Some(interval) = source.interval_ms {
                self.http_due.insert(
                    id,
                    std::time::Instant::now() + std::time::Duration::from_millis(interval),
                );
            }
        }
        let now = std::time::Instant::now();
        if !self.http_started {
            self.http_started = true;
            for (id, source) in &self.description.data_sources {
                if source.on_load {
                    self.http_due.insert(id.clone(), now);
                } else if let Some(ms) = source.interval_ms {
                    self.http_due
                        .insert(id.clone(), now + std::time::Duration::from_millis(ms));
                }
            }
        }
        let due: Vec<_> = self
            .http_due
            .iter()
            .filter(|(_, time)| **time <= now)
            .map(|(id, _)| id.clone())
            .collect();
        for id in due {
            match self.request_http(&id) {
                Ok(true) => {}
                Ok(false) => {
                    self.http_due
                        .insert(id, now + std::time::Duration::from_millis(100));
                }
                Err(e) => {
                    self.error = Some(e);
                    self.http_due
                        .insert(id, now + std::time::Duration::from_secs(1));
                }
            }
        }
    }
    pub fn http_active(&self) -> bool {
        !self.http_jobs.is_empty()
            || self
                .description
                .data_sources
                .values()
                .any(|s| s.on_load || s.interval_ms.is_some())
    }
}
