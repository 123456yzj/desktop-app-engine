use super::Runtime;
use crate::{
    http::{CredentialHeader, HeaderArgument},
    plugin, protocol,
};
use std::collections::BTreeSet;

impl Runtime {
    /// Verify referenced credentials without sending HTTP requests.
    pub fn check_credentials(&self) -> Result<Vec<String>, String> {
        let mut checked = BTreeSet::new();
        for source in self.description.data_sources.values() {
            for header in source.headers.values() {
                if let HeaderArgument::Credential(CredentialHeader::Credential { name, .. }) =
                    header
                    && checked.insert(name.clone())
                {
                    let secret = self
                        .credential_store
                        .as_ref()
                        .ok_or("缺少 credentialStore")?
                        .get(name)?;
                    reqwest::header::HeaderValue::from_str(&secret)
                        .map_err(|_| "凭据无法作为 HTTP 请求头")?;
                }
            }
        }
        Ok(checked.into_iter().collect())
    }

    /// Exercise each registered plugin without exposing runtime internals to the CLI.
    pub fn check_plugins(&self) -> Result<Vec<String>, String> {
        let mut checked = Vec::new();
        for node in &self.tree.nodes {
            if let Some(registration) = &node.metadata.registration {
                let mut rpc = plugin::Rpc::start(registration)?;
                let response = rpc.request(
                    &serde_json::json!({"op":"mount","component":registration.component,
                    "props":protocol::plugin_props(&node.spec,&self.state)?,"width":800,"height":400}),
                )?;
                if let Some(svg) = response.get("svg").and_then(serde_json::Value::as_str) {
                    plugin::rasterize(svg)?;
                }
                rpc.request(
                    &serde_json::json!({"op":"unmount","component":registration.component}),
                )?;
                checked.push(node.spec.id.clone());
            }
        }
        Ok(checked)
    }
}
