use serde::Serialize;

#[derive(Serialize)]
pub struct JWT<'a>(pub(crate) &'a str);
