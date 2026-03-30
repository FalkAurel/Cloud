use fastrace::{Event, Span, prelude::SpanContext};
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::{Method, Status},
    tokio::time::Instant,
};
use std::{borrow::Cow, mem::ManuallyDrop};

struct RequestTrace {
    span: Span,
    start: Instant,
}

trait ToStaticStr {
    fn static_str(&self) -> &'static str;
}

impl ToStaticStr for Method {
    fn static_str(&self) -> &'static str {
        self.as_str()
    }
}

impl ToStaticStr for Status {
    fn static_str(&self) -> &'static str {
        match self.code {
            100 => "100 Continue",
            101 => "101 Switching Protocols",
            102 => "102 Processing",
            200 => "200 OK",
            201 => "201 Created",
            202 => "202 Accepted",
            203 => "203 Non-Authoritative Information",
            204 => "204 No Content",
            205 => "205 Reset Content",
            206 => "206 Partial Content",
            207 => "207 Multi-Status",
            208 => "208 Already Reported",
            226 => "226 IM Used",
            300 => "300 Multiple Choices",
            301 => "301 Moved Permanently",
            302 => "302 Found",
            303 => "303 See Other",
            304 => "304 Not Modified",
            307 => "307 Temporary Redirect",
            308 => "308 Permanent Redirect",
            400 => "400 Bad Request",
            401 => "401 Unauthorized",
            402 => "402 Payment Required",
            403 => "403 Forbidden",
            404 => "404 Not Found",
            405 => "405 Method Not Allowed",
            406 => "406 Not Acceptable",
            408 => "408 Request Timeout",
            409 => "409 Conflict",
            410 => "410 Gone",
            411 => "411 Length Required",
            412 => "412 Precondition Failed",
            413 => "413 Payload Too Large",
            414 => "414 URI Too Long",
            415 => "415 Unsupported Media Type",
            416 => "416 Range Not Satisfiable",
            417 => "417 Expectation Failed",
            418 => "418 I'm a teapot",
            422 => "422 Unprocessable Entity",
            429 => "429 Too Many Requests",
            500 => "500 Internal Server Error",
            501 => "501 Not Implemented",
            502 => "502 Bad Gateway",
            503 => "503 Service Unavailable",
            504 => "504 Gateway Timeout",
            _ => "??? Unknown",
        }
    }
}

pub struct RequestTraceSpan;

impl RequestTraceSpan {
    pub fn new() -> Self {
        Self
    }
}

impl Fairing for RequestTraceSpan {
    fn info(&self) -> Info {
        Info {
            name: "Request Tracer",
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
        let method: &'static str = req.method().as_str();
        let path: &str = req.uri().path().as_str();

        let span: Span = Span::root(format!("HTTP {} {}", method, path), SpanContext::random())
            .with_property(move || ("http.method", Cow::Borrowed(method))); // &'static str, no alloc

        req.local_cache(|| {
            ManuallyDrop::new(RequestTrace {
                span,
                start: Instant::now(),
            })
        });

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
        let trace: &ManuallyDrop<RequestTrace> = req.local_cache(|| {
            ManuallyDrop::new(RequestTrace {
                span: Span::noop(),
                start: Instant::now(),
            })
        });

        trace.span.add_event(
            Event::new("request.complete")
                .with_property(|| ("http.status", res.status().static_str()))
                .with_property(|| ("duration_ms", trace.start.elapsed().as_millis().to_string())),
        );

        Box::pin(async {})
    }
}
