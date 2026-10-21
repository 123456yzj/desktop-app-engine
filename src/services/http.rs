//! Bounded HTTP JSON data sources. Workers never mutate runtime state.
use crate::{
    binding::{Binding, State, ValueType, valid_pointer},
    command::Argument,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::BTreeMap, io::Read, time::Duration};
pub const MAX_RESPONSE: usize = 4 * 1024 * 1024;
pub fn default_timeout() -> u64 {
    15_000
}
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum CredentialHeader {
    Credential {
        name: String,
        #[serde(default)]
        prefix: String,
    },
}
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum HeaderArgument {
    Text(String),
    Binding(Binding),
    Credential(CredentialHeader),
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Mapping {
    pub target: String,
    #[serde(default)]
    pub path: String,
}
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub enum Method {
    #[default]
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
}
impl Method {
    fn value(self) -> reqwest::Method {
        match self {
            Self::GET => reqwest::Method::GET,
            Self::POST => reqwest::Method::POST,
            Self::PUT => reqwest::Method::PUT,
            Self::PATCH => reqwest::Method::PATCH,
            Self::DELETE => reqwest::Method::DELETE,
        }
    }
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Source {
    pub url: Argument,
    #[serde(default)]
    pub method: Method,
    #[serde(default)]
    pub headers: BTreeMap<String, HeaderArgument>,
    #[serde(default)]
    pub query: BTreeMap<String, Argument>,
    pub body: Option<Binding>,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    pub status_target: String,
    pub response_sample: Value,
    pub mappings: Vec<Mapping>,
    #[serde(default)]
    pub on_load: bool,
    pub interval_ms: Option<u64>,
    pub enabled: Option<Binding>,
}
pub struct Request {
    url: reqwest::Url,
    method: reqwest::Method,
    headers: reqwest::header::HeaderMap,
    query: Vec<(String, String)>,
    body: Option<Value>,
    timeout: Duration,
    credentials: Vec<(reqwest::header::HeaderName, String, String)>,
    store: Option<crate::credentials::Store>,
}
pub struct Response {
    pub code: u16,
    pub result: Result<Value, String>,
}
pub fn status(phase: &str, loading: bool, code: u16, error: &str, updated_at: u64) -> Value {
    json!({"status":phase,"loading":loading,"statusCode":code,"error":error,"updatedAt":updated_at})
}
pub fn initial_status() -> Value {
    status("idle", false, 0, "", 0)
}
impl Source {
    pub fn bindings_mut(&mut self) -> Vec<&mut Binding> {
        let mut result = Vec::new();
        for arg in std::iter::once(&mut self.url).chain(self.query.values_mut()) {
            if let Argument::Binding(b) = arg {
                result.push(b);
            }
        }
        for header in self.headers.values_mut() {
            if let HeaderArgument::Binding(b) = header {
                result.push(b);
            }
        }
        result.extend(self.body.iter_mut());
        result.extend(self.enabled.iter_mut());
        result
    }
    pub fn is_enabled(&self, state: &State) -> Result<bool, String> {
        self.enabled
            .as_ref()
            .map(|b| {
                b.resolve(state)?
                    .as_bool()
                    .ok_or("dataSource.enabled 必须为布尔值".into())
            })
            .unwrap_or(Ok(true))
    }
    pub fn resolve(&self, state: &State) -> Result<Request, String> {
        self.resolve_with_store(state, None)
    }
    pub fn requires_credentials(&self) -> bool {
        self.headers
            .values()
            .any(|h| matches!(h, HeaderArgument::Credential(_)))
    }
    pub fn resolve_with_store(
        &self,
        state: &State,
        store: Option<crate::credentials::Store>,
    ) -> Result<Request, String> {
        let url = reqwest::Url::parse(&self.url.resolve(state)?)
            .map_err(|_| "接口 URL 无效".to_owned())?;
        if !["http", "https"].contains(&url.scheme())
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err("接口 URL 必须为 HTTP(S)，认证请使用 headers".into());
        }
        let mut headers = reqwest::header::HeaderMap::new();
        let mut credentials = Vec::new();
        for (key, arg) in &self.headers {
            let name = reqwest::header::HeaderName::from_bytes(key.as_bytes())
                .map_err(|_| format!("非法请求头名称 {key:?}"))?;
            if ["content-length", "transfer-encoding", "host"].contains(&name.as_str()) {
                return Err("Host/Content-Length/Transfer-Encoding 由客户端管理".into());
            }
            if [
                "authorization",
                "proxy-authorization",
                "cookie",
                "x-api-key",
                "api-key",
                "x-auth-token",
            ]
            .contains(&name.as_str())
                && !matches!(arg, HeaderArgument::Credential(_))
            {
                return Err(format!(
                    "认证请求头 {key:?} 必须引用加密凭据，不能使用明文或 state"
                ));
            }
            let text = match arg {
                HeaderArgument::Text(text) => text.clone(),
                HeaderArgument::Binding(b) => b
                    .resolve(state)?
                    .as_str()
                    .ok_or("HTTP 请求头绑定必须为字符串")?
                    .to_owned(),
                HeaderArgument::Credential(CredentialHeader::Credential {
                    name: credential,
                    prefix,
                }) => {
                    crate::credentials::valid_name(credential)?;
                    if prefix.len() > 128
                        || reqwest::header::HeaderValue::from_str(&format!("{prefix}placeholder"))
                            .is_err()
                    {
                        return Err("凭据请求头 prefix 无效或超限".into());
                    }
                    credentials.push((name, credential.clone(), prefix.clone()));
                    continue;
                }
            };
            let value = reqwest::header::HeaderValue::from_str(&text)
                .map_err(|_| format!("非法请求头值 {key:?}"))?;
            headers.insert(name, value);
        }
        let query = self
            .query
            .iter()
            .map(|(key, arg)| Ok((key.clone(), arg.resolve(state)?)))
            .collect::<Result<_, String>>()?;
        let body = self
            .body
            .as_ref()
            .map(|b| b.resolve(state).cloned())
            .transpose()?;
        if matches!(self.method, Method::GET) && body.is_some() {
            return Err("GET 不接受 body".into());
        }
        if body
            .as_ref()
            .is_some_and(|v| v.to_string().len() > MAX_RESPONSE)
        {
            return Err("接口请求 body 超过 4MiB".into());
        }
        Ok(Request {
            url,
            method: self.method.value(),
            headers,
            query,
            body,
            timeout: Duration::from_millis(self.timeout_ms),
            credentials,
            store,
        })
    }
    pub fn validate(&self, state: &State) -> Result<(), String> {
        if !(1..=120_000).contains(&self.timeout_ms) {
            return Err("timeoutMs 必须在 1..120000".into());
        }
        if self
            .interval_ms
            .is_some_and(|v| !(100..=86_400_000).contains(&v))
        {
            return Err("intervalMs 必须在 100..86400000".into());
        }
        self.is_enabled(state)?;
        self.resolve(state)?;
        let initial = state
            .get(&self.status_target)
            .ok_or("statusTarget 状态不存在")?;
        if ValueType::of(&initial_status())? != ValueType::of(initial)? {
            return Err("statusTarget 结构必须为 status/loading/statusCode/error/updatedAt".into());
        }
        if self.mappings.is_empty() {
            return Err("mappings 不得为空".into());
        }
        let mut targets = std::collections::HashSet::new();
        targets.insert(&self.status_target);
        for m in &self.mappings {
            if !valid_pointer(&m.path) || !targets.insert(&m.target) {
                return Err("非法 mapping path 或重复目标状态".into());
            }
            let value = self
                .response_sample
                .pointer(&m.path)
                .ok_or("responseSample 缺少 mapping path")?;
            let initial = state.get(&m.target).ok_or("mapping target 状态不存在")?;
            ValueType::of(value)?;
            if !ValueType::of(initial)?.accepts(value) {
                return Err(format!("mapping {:?}: 响应样本与状态类型不匹配", m.target));
            }
        }
        Ok(())
    }
    pub fn map(&self, value: &Value) -> Result<BTreeMap<String, Value>, String> {
        self.mappings
            .iter()
            .map(|m| {
                Ok((
                    m.target.clone(),
                    value
                        .pointer(&m.path)
                        .ok_or_else(|| format!("响应缺少字段 {:?}", m.path))?
                        .clone(),
                ))
            })
            .collect()
    }
}
pub fn run(mut request: Request) -> Response {
    let mut code = 0;
    let result = (|| {
        for (header, name, prefix) in request.credentials {
            let store = request.store.as_ref().ok_or("请求缺少 credentialStore")?;
            let secret = store.get(&name)?;
            let mut text = zeroize::Zeroizing::new(prefix);
            text.push_str(&secret);
            let mut value = reqwest::header::HeaderValue::from_str(&text)
                .map_err(|_| "凭据无法作为 HTTP 请求头")?;
            value.set_sensitive(true);
            request.headers.insert(header, value);
        }
        let client = reqwest::blocking::Client::builder()
            .timeout(request.timeout)
            .connect_timeout(request.timeout)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| "HTTP 客户端初始化失败".to_owned())?;
        let mut builder = client
            .request(request.method, request.url)
            .headers(request.headers)
            .query(&request.query);
        if let Some(body) = request.body {
            builder = builder.json(&body);
        }
        let response = builder.send().map_err(|e| {
            if e.is_timeout() {
                "请求超时"
            } else if e.is_connect() {
                "连接失败"
            } else {
                "请求发送失败"
            }
            .to_owned()
        })?;
        code = response.status().as_u16();
        if !(200..300).contains(&code) {
            return Err(format!("HTTP {code}"));
        }
        if response
            .content_length()
            .is_some_and(|n| n > MAX_RESPONSE as u64)
        {
            return Err("响应超过 4MiB".into());
        }
        let mut bytes = Vec::new();
        response
            .take(MAX_RESPONSE as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "响应读取失败或超时".to_owned())?;
        if bytes.len() > MAX_RESPONSE {
            return Err("响应超过 4MiB".into());
        }
        serde_json::from_slice(&bytes).map_err(|_| "响应不是有效 JSON".into())
    })();
    Response { code, result }
}
