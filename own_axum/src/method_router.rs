use super::http_method::HttpMethod;
use super::types::BoxedHandler;
use std::collections::HashMap;

#[derive(Default)]
pub struct MethodRouter {
    pub(crate) handlers: HashMap<HttpMethod, BoxedHandler>,
}

impl MethodRouter {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub(crate) fn get(mut self, handler: BoxedHandler) -> Self {
        self.handlers.insert(HttpMethod::GET, handler);
        self
    }

    pub fn put(mut self, handler: BoxedHandler) -> Self {
        self.handlers.insert(HttpMethod::PUT, handler);
        self
    }

    pub(crate) fn post(mut self, handler: BoxedHandler) -> Self {
        self.handlers.insert(HttpMethod::POST, handler);
        self
    }
}

pub fn get(handler: BoxedHandler) -> MethodRouter {
    MethodRouter::new().get(handler)
}

pub fn post(handler: BoxedHandler) -> MethodRouter {
    MethodRouter::new().post(handler)
}
