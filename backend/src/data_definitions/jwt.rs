use std::{
    env,
    fmt::Debug,
    sync::LazyLock,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chrono::DateTime;
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, decode_header, encode,
    errors::Error as JWTError,
};
use rocket::{
    http::{Cookie, Status},
    request::{FromRequest, Outcome},
};
use serde::{Deserialize, Serialize};

static JWT_SECRET: LazyLock<EncodingKey> = LazyLock::new(|| {
    let secret: String = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    EncodingKey::from_secret(secret.as_bytes())
});

#[derive(Serialize, Deserialize)]
pub struct JWT {
    pub(crate) exp: u64,     // in milis
    pub(crate) user_id: u32, // 16K users should be fine for now
}

impl Debug for JWT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UserID: {}, Expiration: {}",
            self.user_id,
            DateTime::from_timestamp_millis(self.exp as i64).unwrap_or_else(|| DateTime::default()) // ToDo: is this really the way to handle the case when we face an invalid expiration date?
        )
    }
}

#[derive(Debug)]
pub enum DecodeError {
    Expired,
    JWTError(JWTError),
}

impl JWT {
    pub fn create(user_id: u32, duration: Duration) -> Result<String, JWTError> {
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
        let header: Header =
            decode_header(cookie_claim).map_err(|err| DecodeError::JWTError(err))?;

        match decode::<JWT>(
            cookie_claim,
            &DecodingKey::from_secret(JWT_SECRET.inner()),
            &Validation::new(header.alg),
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
    fn create_jwt() {
        let jwt: String = JWT::create(0, Duration::from_mins(10)).unwrap();
        let JWT { user_id, .. } = JWT::decode(&jwt).unwrap();
        assert_eq!(user_id, 0);
    }

    #[test]
    fn expired_jwt() {
        let jwt: String = JWT::create(0, Duration::from_micros(100)).unwrap();
        sleep(Duration::from_millis(1));

        assert!(JWT::decode(&jwt).is_err())
    }
}
