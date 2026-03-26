use rocket::{
    fairing::{Fairing, Info, Kind},
    tokio::time::Instant,
};
use tracing::info;

pub struct HttpSpan;

pub struct RequestTraceSpan;

impl RequestTraceSpan {
    pub fn new() -> Self {
        Self
    }
}

impl Fairing for RequestTraceSpan {
    fn info(&self) -> Info {
        Info {
            name: "Request Logger",
            kind: Kind::Request | Kind::Response,
        }
    }

    fn on_request<'life0, 'life1, 'life2, 'life3, 'life4, 'async_trait>(
        &'life0 self,
        req: &'life1 mut rocket::Request<'life2>,
        _data: &'life3 mut rocket::Data<'life4>,
    ) -> std::pin::Pin<
        Box<dyn ::core::future::Future<Output = ()> + ::core::marker::Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        'life3: 'async_trait,
        'life4: 'async_trait,
        Self: 'async_trait,
    {
        let method: &str = req.method().as_str();
        let path: &str = req.uri().path().as_str();

        info!("--> {} {}", method, path);

        req.local_cache(|| Instant::now());
        Box::pin(async {})
    }

    fn on_response<'r, 'life0, 'life1, 'life2, 'async_trait>(
        &'life0 self,
        req: &'r rocket::Request<'life1>,
        res: &'life2 mut rocket::Response<'r>,
    ) -> std::pin::Pin<
        Box<dyn ::core::future::Future<Output = ()> + ::core::marker::Send + 'async_trait>,
    >
    where
        'r: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        Self: 'async_trait,
    {
        let method: &str = req.method().as_str();
        let path: &str = req.uri().path().as_str();
        let status: u16 = res.status().code;
        let elapsed: std::time::Duration = Instant::now() - *req.local_cache(|| Instant::now());

        info!("<-- {} {} {} ({:.2?})", status, method, path, elapsed);

        Box::pin(async {})
    }
}
