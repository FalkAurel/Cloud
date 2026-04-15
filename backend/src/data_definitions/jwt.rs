//! # JWT authentication
//!
//! Tokens are signed with HMAC-SHA256 (`HS256`) and stored in `HttpOnly` cookies.
//! The signing algorithm is locked at decode time and never read from the token
//! header, preventing algorithm-confusion attacks.
//!
//! ## Why `exp` is in milliseconds
//!
//! [`JWT::exp`] stores the expiry as **milliseconds since the Unix epoch**, not
//! the seconds that the JWT spec (RFC 7519) requires.  This lets the application
//! use sub-second lifetimes in tests without floating-point arithmetic.
//!
//! Consequences to be aware of:
//! - [`jsonwebtoken`]'s built-in `exp` validator effectively becomes a no-op
//!   because a millisecond timestamp (~1.7 × 10¹²) is always greater than the
//!   current Unix timestamp in seconds (~1.7 × 10⁹).  Expiry enforcement is
//!   handled entirely by the manual check in [`JWT::decode`].
//! - External tools (e.g. jwt.io) will display the expiry as a date far in the
//!   future.  This is expected and not a bug.
//!
//! If you change this to seconds, update [`JWT::create`], [`JWT::decode`], and
//! the tests in this module.

use std::{
    env,
    fmt::Debug,
    sync::LazyLock,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chrono::DateTime;
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode,
    errors::Error as JWTError,
};
use rocket::{
    http::{Cookie, Status},
    request::{FromRequest, Outcome},
};
use serde::{Deserialize, Serialize};

/// Minimum number of bytes required for the JWT signing secret.
/// 32 bytes = 256 bits — the minimum recommended for HMAC-SHA256.
const JWT_SECRET_MIN_BYTES: usize = 32;

static JWT_SECRET: LazyLock<EncodingKey> = LazyLock::new(|| {
    let secret: String = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    assert!(
        secret.len() >= JWT_SECRET_MIN_BYTES,
        "JWT_SECRET must be at least {JWT_SECRET_MIN_BYTES} bytes (got {}). \
         Generate one with: openssl rand -hex 32",
        secret.len()
    );
    EncodingKey::from_secret(secret.as_bytes())
});

#[derive(Serialize, Deserialize)]
pub struct JWT {
    pub(crate) exp: u64, // in millis — see module doc for rationale
    pub(crate) user_id: i32,
}

impl Debug for JWT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let expiration = DateTime::from_timestamp_millis(self.exp as i64)
            .map(|dt| dt.to_string())
            .unwrap_or_else(|| "<invalid timestamp>".to_string());
        write!(f, "UserID: {}, Expiration: {}", self.user_id, expiration)
    }
}

#[derive(Debug)]
pub enum DecodeError {
    Expired,
    JWTError(JWTError),
}

impl JWT {
    pub fn create(user_id: i32, duration: Duration) -> Result<String, JWTError> {
        let now: Duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Should never fail");

        let claims: JWT = JWT {
            exp: (now + duration).as_millis() as u64,
            user_id,
        };

        encode(&Header::new(Algorithm::HS256), &claims, &JWT_SECRET)
    }

    fn decode(cookie_claim: &str) -> Result<JWT, DecodeError> {
        // Disable the library's built-in exp check: our `exp` is in milliseconds
        // (see module doc), so the library would never reject a token since a
        // millisecond timestamp is always >> the current Unix timestamp in seconds.
        // Expiry is enforced manually below.
        let mut validation: Validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false;

        match decode::<JWT>(
            cookie_claim,
            &DecodingKey::from_secret(JWT_SECRET.inner()),
            &validation,
        ) {
            Ok(data) => {
                let now: Duration = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Should never fail");
                if now > Duration::from_millis(data.claims.exp) {
                    return Err(DecodeError::Expired);
                }
                Ok(data.claims)
            }
            Err(err) => Err(DecodeError::JWTError(err)),
        }
    }
}

#[derive(Debug)]
pub struct Auth(pub JWT);

impl Auth {
    pub fn get_jwt(self) -> JWT {
        self.0
    }
}

#[derive(Debug)]
pub enum AuthError {
    JWTError(JWTError),
    NoJWT,
    Expired,
}

impl<'a> FromRequest<'a> for Auth {
    type Error = AuthError;

    fn from_request<'life0, 'async_trait>(
        request: &'a rocket::Request<'life0>,
    ) -> core::pin::Pin<
        Box<dyn ::core::future::Future<Output = Outcome<Self, Self::Error>> + Send + 'async_trait>,
    >
    where
        'a: 'async_trait,
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        let cookie: &Cookie<'_> = match request.cookies().get("jwt") {
            Some(cookie) => cookie,
            None => {
                return Box::pin(async {
                    Outcome::Error((Status::Unauthorized, AuthError::NoJWT))
                });
            }
        };

        match JWT::decode(cookie.value()) {
            Ok(claim) => Box::pin(async { Outcome::Success(Auth(claim)) }),
            Err(err) => match err {
                DecodeError::Expired => {
                    Box::pin(async { Outcome::Error((Status::Unauthorized, AuthError::Expired)) })
                }
                DecodeError::JWTError(err) => Box::pin(async {
                    Outcome::Error((Status::BadRequest, AuthError::JWTError(err)))
                }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use crate::data_definitions::JWT;

    #[test]
    #[ignore = "requires JWT_SECRET env var"]
    fn create_jwt() {
        let jwt: String = JWT::create(0, Duration::from_mins(10)).unwrap();
        let JWT { user_id, .. } = JWT::decode(&jwt).unwrap();
        assert_eq!(user_id, 0);
    }

    #[test]
    #[ignore = "requires JWT_SECRET env var"]
    fn expired_jwt() {
        let jwt: String = JWT::create(0, Duration::from_micros(100)).unwrap();
        sleep(Duration::from_millis(1));

        assert!(JWT::decode(&jwt).is_err())
    }
}
