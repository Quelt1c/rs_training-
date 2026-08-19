use crate::http_request::HttpRequest;
use crate::http_response::HttpResponse;
use crate::types::{BoxFuture, BoxedHandler};
use std::future::Future;
use std::sync::Arc;

pub struct Next {
    inner: BoxedHandler,
}

impl Next {
    pub async fn run(self, req: HttpRequest) -> HttpResponse {
        (self.inner)(req).await
    }
}

pub fn from_fn<F, Fut>(mw: F) -> impl Fn(BoxedHandler) -> BoxedHandler
where
    F: Fn(HttpRequest, Next) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = HttpResponse> + Send + 'static,
{
    move |inner: BoxedHandler| {
        let mw = mw.clone();

        Arc::new(move |req: HttpRequest| {
            let mw = mw.clone();
            let next = Next {
                inner: Arc::clone(&inner),
            };

            Box::pin(async move { mw(req, next).await }) as BoxFuture
        })
    }
}
