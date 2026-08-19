use super::content_type::ContentType;
use super::http_method::HttpMethod;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    method: HttpMethod,
    path: String,
    raw_query: String,
    query: HashMap<String, String>,
    headers: HashMap<String, String>,
    body: String,
}

impl HttpRequest {
    pub(crate) fn new(
        method: HttpMethod,
        path: String,
        raw_query: String,
        query: HashMap<String, String>,
        headers: HashMap<String, String>,
        body: String,
    ) -> Self {
        Self {
            method,
            path,
            raw_query,
            query,
            headers,
            body,
        }
    }

    pub fn method(&self) -> HttpMethod {
        self.method
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn raw_query(&self) -> &str {
        &self.raw_query
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(&name.to_lowercase()).map(String::as_str)
    }

    pub fn query_param(&self, name: &str) -> Option<&str> {
        self.query.get(name).map(String::as_str)
    }

    pub fn content_type(&self) -> Option<ContentType> {
        self.header("content-type")
            .and_then(ContentType::from_header)
    }
}
