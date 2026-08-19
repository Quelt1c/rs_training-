use super::http_method::HttpMethod;
use serde::de::DeserializeOwned;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    method: HttpMethod,
    path: String,
    raw_query: String,
    headers: HashMap<String, String>,
    body: String,
    path_params: HashMap<String, String>,
    cookies: HashMap<String, String>,
}

fn parse_cookies(header_value: &str) -> HashMap<String, String> {
    header_value
        .split(';')
        .filter_map(|pair| {
            let (key, value) = pair.trim().split_once('=')?;
            let key = key.trim();
            if key.is_empty() {
                None
            } else {
                Some((key.to_string(), value.trim().to_string()))
            }
        })
        .collect()
}

impl HttpRequest {
    pub(crate) fn new(
        method: HttpMethod,
        path: String,
        raw_query: String,
        headers: HashMap<String, String>,
        body: String,
    ) -> Self {
        let cookies = headers
            .get("cookie")
            .map(|value| parse_cookies(value))
            .unwrap_or_default();

        Self {
            method,
            path,
            raw_query,
            headers,
            body,
            path_params: HashMap::new(),
            cookies,
        }
    }

    pub(crate) fn set_path_params(&mut self, params: HashMap<String, String>) {
        self.path_params = params;
    }

    pub fn method(&self) -> HttpMethod {
        self.method
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(&name.to_lowercase()).map(String::as_str)
    }

    pub fn cookie(&self, name: &str) -> Option<&str> {
        self.cookies.get(name).map(String::as_str)
    }

    pub fn path_param(&self, name: &str) -> Option<&str> {
        self.path_params.get(name).map(String::as_str)
    }

    pub fn query<T: DeserializeOwned>(&self) -> Result<T, serde_urlencoded::de::Error> {
        serde_urlencoded::from_str(&self.raw_query)
    }
}
