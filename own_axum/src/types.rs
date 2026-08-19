use super::http_request::HttpRequest;
use super::http_response::HttpResponse;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub(crate) type BoxFuture = Pin<Box<dyn Future<Output = HttpResponse> + Send>>;
pub type BoxedHandler = Arc<dyn Fn(HttpRequest) -> BoxFuture + Send + Sync>;
