use std::sync::LazyLock;

use rocket::fairing::{Fairing, Info};
use tracing::Level;
use tracing_subscriber::EnvFilter;

const TRACE_LEVEL: LazyLock<Level> = LazyLock::new(|| {
    EnvFilter::from_default_env().max_level_hint().expect("Failed to get max level hint").into_level().unwrap()
});
pub struct RequestTraceSpan;

impl Fairing for RequestTraceSpan {
    fn info(&self) -> rocket::fairing::Info {
        Info {
            name: "Request Trace Span",
            kind: rocket::fairing::Kind::Request | rocket::fairing::Kind::Response,
        }
    }

    fn on_request<'life0,'life1,'life2,'life3,'life4,'async_trait>(&'life0 self,_req: &'life1 mut rocket::Request<'life2> ,_data: &'life3 mut rocket::Data<'life4>) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = ()> + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,'life1:'async_trait,'life2:'async_trait,'life3:'async_trait,'life4:'async_trait,Self:'async_trait {
        let a = EnvFilter::from_default_env().max_level_hint().expect("Nice").into_level().unwrap();
        tracing::span!()
    }
}