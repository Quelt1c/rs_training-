use super::http_request::HttpRequest;
use super::http_response::HttpResponse;
use super::types::{BoxFuture, BoxedHandler};
use std::future::Future;
use std::sync::Arc;

pub fn with_state<F, Fut, S>(f: F, state: S) -> BoxedHandler
where
    F: Fn(HttpRequest, S) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = HttpResponse> + Send + 'static,
    S: Clone + Send + Sync + 'static,
{
    let f = Arc::new(f);

    Arc::new(move |req: HttpRequest| {
        let state = state.clone();
        let f = Arc::clone(&f);

        Box::pin(async move { f(req, state).await }) as BoxFuture
    })
}
