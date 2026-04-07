#[cfg(feature = "email")]
mod email;

#[cfg(feature = "email")]
pub(crate) use email::Email;

#[cfg(feature = "email")]
pub use email::{EmailError, EmailSender, init_email_sender};

mod fixed_len_str;
mod jwt;
mod user;

pub use fixed_len_str::FixedSizedStr;
pub use jwt::{Auth, JWT};
pub(crate) use user::{MAX_UTF8_BYTES, UserLoginView};
pub use user::{StandardUserView, UserLoginRequest, UserSignupRequest};
