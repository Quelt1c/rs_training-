use super::http_request::HttpRequest;
use super::http_response::HttpResponse;
use super::status_code::StatusCode;
use super::types::{BoxFuture, BoxedHandler};
use serde::de::DeserializeOwned;
use std::future::Future;
use std::sync::Arc;

pub struct Query<T>(pub T);

impl<T: DeserializeOwned> Query<T> {
    pub(crate) fn extract(req: &HttpRequest) -> Result<Self, HttpResponse> {
        serde_urlencoded::from_str::<T>(req.raw_query())
            .map(Query)
            .map_err(|e| {
                HttpResponse::new(
                    StatusCode::BAD_REQUEST,
                    &format!("Invalid query parameters: {e}"),
                )
            })
    }
}

pub fn with_query_handler<F, Fut, S, R, T>(f: F, state: S) -> BoxedHandler
where
    F: Fn(HttpRequest, Query<T>, S) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: Into<HttpResponse>,
    S: Clone + Send + Sync + 'static,
    T: DeserializeOwned + Send + Sync + 'static,
{
    let f = Arc::new(f);

    Box::new(move |req: HttpRequest| {
        let parsed_query = Query::<T>::extract(&req);
        let state = state.clone();
        let f = Arc::clone(&f);

        Box::pin(async move {
            match parsed_query {
                Ok(query) => f(req, query, state).await.into(),
                Err(bad_request) => bad_request,
            }
        }) as BoxFuture
    })
}
