use super::{Result, ScriptHostRuntimeError};
use crate::policy::ScriptHttpPolicy;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HttpRequestPlan {
    pub method: String,
    pub url: String,
    pub body: Option<String>,
    pub headers: Vec<(String, String)>,
    pub content_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HttpResponseRecord {
    pub status_code: u16,
    pub headers: BTreeMap<String, String>,
    pub body: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum HttpDispatchMode {
    PlanOnly,
    Reqwest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HttpExecution {
    pub mode: HttpDispatchMode,
    pub request: HttpRequestPlan,
    pub response: Option<HttpResponseRecord>,
    pub dispatched: bool,
}

pub trait ScriptHttpClient {
    fn send(&mut self, request: HttpRequestPlan) -> Result<HttpResponseRecord>;
}

#[derive(Debug, Clone)]
pub struct RecordingHttpClient {
    response: HttpResponseRecord,
    requests: Vec<HttpRequestPlan>,
}

impl RecordingHttpClient {
    pub fn new(response: HttpResponseRecord) -> Self {
        Self {
            response,
            requests: Vec::new(),
        }
    }

    pub fn ok_json(body: impl Into<String>) -> Self {
        Self::new(HttpResponseRecord {
            status_code: 200,
            headers: BTreeMap::from([("content-type".to_string(), "application/json".to_string())]),
            body: body.into(),
        })
    }

    pub fn requests(&self) -> &[HttpRequestPlan] {
        &self.requests
    }
}

impl ScriptHttpClient for RecordingHttpClient {
    fn send(&mut self, request: HttpRequestPlan) -> Result<HttpResponseRecord> {
        self.requests.push(request);
        Ok(self.response.clone())
    }
}

#[derive(Debug, Clone)]
pub struct ReqwestScriptHttpClient {
    client: reqwest::blocking::Client,
}

impl ReqwestScriptHttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl Default for ReqwestScriptHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptHttpClient for ReqwestScriptHttpClient {
    fn send(&mut self, request: HttpRequestPlan) -> Result<HttpResponseRecord> {
        let method = request
            .method
            .parse::<reqwest::Method>()
            .map_err(|_| ScriptHostRuntimeError::InvalidHttpMethod(request.method.clone()))?;
        let mut builder = self.client.request(method, &request.url);
        for (key, value) in request.headers {
            builder = builder.header(key, value);
        }
        if let Some(body) = request.body {
            builder = builder
                .header(reqwest::header::CONTENT_TYPE, request.content_type)
                .body(body);
        }

        let response = builder.send()?;
        let status_code = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .filter_map(|(key, value)| {
                value
                    .to_str()
                    .ok()
                    .map(|value| (key.as_str().to_string(), value.to_string()))
            })
            .collect::<BTreeMap<_, _>>();
        let body = response.text()?;

        Ok(HttpResponseRecord {
            status_code,
            headers,
            body,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HttpHost {
    policy: ScriptHttpPolicy,
}

impl HttpHost {
    pub fn new(policy: ScriptHttpPolicy) -> Self {
        Self { policy }
    }

    pub fn policy(&self) -> &ScriptHttpPolicy {
        &self.policy
    }

    pub fn request(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
        headers_json: Option<&str>,
    ) -> Result<HttpRequestPlan> {
        self.policy.check_url(url)?;
        let (headers, content_type) = normalize_http_headers(headers_json)?;
        Ok(HttpRequestPlan {
            method: method.to_ascii_uppercase(),
            url: url.to_string(),
            body: body.map(ToOwned::to_owned),
            headers,
            content_type,
        })
    }

    pub fn execute_request<C: ScriptHttpClient>(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
        headers_json: Option<&str>,
        client: &mut C,
    ) -> Result<HttpResponseRecord> {
        let plan = self.request(method, url, body, headers_json)?;
        client.send(plan)
    }
}

fn normalize_http_headers(headers_json: Option<&str>) -> Result<(Vec<(String, String)>, String)> {
    let mut headers = Vec::new();
    let mut content_type = "application/json".to_string();
    let Some(headers_json) = headers_json else {
        return Ok((headers, content_type));
    };
    if headers_json.trim().is_empty() {
        return Ok((headers, content_type));
    }

    let value: Value = serde_json::from_str(headers_json)
        .map_err(|_| ScriptHostRuntimeError::InvalidHttpHeaders)?;
    let Value::Object(map) = value else {
        return Err(ScriptHostRuntimeError::InvalidHttpHeaders);
    };

    for (key, value) in map {
        let Some(value) = value.as_str() else {
            return Err(ScriptHostRuntimeError::InvalidHttpHeaders);
        };
        let key = key.to_ascii_lowercase();
        if key == "content-type" {
            content_type = value.to_string();
        } else {
            headers.push((key, value.to_string()));
        }
    }
    headers.sort();
    Ok((headers, content_type))
}
