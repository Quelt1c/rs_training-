use super::http_method::HttpMethod;
use super::http_request::HttpRequest;
use super::method_router::MethodRouter;
use super::types::{BoxFuture, BoxedHandler};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

#[derive(Clone)]
enum Segment {
    Literal(String),
    Param(String),
}

struct Route {
    method: HttpMethod,
    segments: Vec<Segment>,
    handler: BoxedHandler,
}

pub(crate) enum RouteMatch {
    Matched(BoxedHandler, HashMap<String, String>),
    MethodNotAllowed,
    NotFound,
}

fn parse_segments(path: &str) -> Vec<Segment> {
    path.split('/')
        .filter(|s| !s.is_empty())
        .map(|s| match s.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
            Some(name) => Segment::Param(name.to_string()),
            None => Segment::Literal(s.to_string()),
        })
        .collect()
}

fn match_segments(segments: &[Segment], path: &str) -> Option<HashMap<String, String>> {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if parts.len() != segments.len() {
        return None;
    }

    let mut params = HashMap::new();

    for (segment, part) in segments.iter().zip(parts.iter()) {
        match segment {
            Segment::Literal(literal) => {
                if literal != part {
                    return None;
                }
            }
            Segment::Param(name) => {
                params.insert(name.clone(), part.to_string());
            }
        }
    }

    Some(params)
}

pub struct Router {
    routes: Vec<Route>,
    pub(crate) fallback_handler: Option<BoxedHandler>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            fallback_handler: None,
        }
    }

    pub fn route(mut self, path: &str, method_router: MethodRouter) -> Self {
        let segments = parse_segments(path);

        for (method, handler) in method_router.handlers {
            self.routes.push(Route {
                method,
                segments: segments.clone(),
                handler,
            });
        }

        self
    }

    pub(crate) fn find(&self, method: HttpMethod, path: &str) -> RouteMatch {
        let mut path_shape_matches = false;

        for route in &self.routes {
            if let Some(params) = match_segments(&route.segments, path) {
                if route.method == method {
                    return RouteMatch::Matched(route.handler.clone(), params);
                }
                path_shape_matches = true;
            }
        }

        if path_shape_matches {
            RouteMatch::MethodNotAllowed
        } else {
            RouteMatch::NotFound
        }
    }

    pub fn fallback<F, Fut, R>(mut self, handler: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: Into<super::http_response::HttpResponse>,
    {
        self.fallback_handler = Some(Arc::new(move |_req: HttpRequest| {
            let fut = handler();
            Box::pin(async move { fut.await.into() }) as BoxFuture
        }));
        self
    }
}
