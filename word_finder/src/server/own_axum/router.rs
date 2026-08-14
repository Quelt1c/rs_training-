use super::http_method::HttpMethod;
use super::http_request::HttpRequest;
use super::types::{BoxFuture, BoxedHandler};
use std::collections::HashMap;
use std::future::Future;

pub struct Router {
    pub(crate) routes: HashMap<(HttpMethod, String), BoxedHandler>,
    pub(crate) fallback_handler: Option<BoxedHandler>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            fallback_handler: None,
        }
    }

    pub fn route(self, path: &str, handler: BoxedHandler) -> Self {
        self.get(path, handler)
    }

    pub fn get(mut self, path: &str, handler: BoxedHandler) -> Self {
        self.routes
            .insert((HttpMethod::GET, path.to_string()), handler);
        self
    }

    pub fn post(mut self, path: &str, handler: BoxedHandler) -> Self {
        self.routes
            .insert((HttpMethod::POST, path.to_string()), handler);
        self
    }

    pub fn fallback<F, Fut, R>(mut self, handler: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: Into<super::http_response::HttpResponse>,
    {
        self.fallback_handler = Some(Box::new(move |_req: HttpRequest| {
            let fut = handler();
            Box::pin(async move { fut.await.into() }) as BoxFuture
        }));
        self
    }
}
