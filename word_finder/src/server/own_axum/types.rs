use super::http_request::HttpRequest;
use super::http_response::HttpResponse;
use std::future::Future;
use std::pin::Pin;

pub(crate) type BoxFuture = Pin<Box<dyn Future<Output = HttpResponse> + Send>>;
pub type BoxedHandler = Box<dyn Fn(HttpRequest) -> BoxFuture + Send + Sync>;
