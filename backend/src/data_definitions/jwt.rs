use std::{
    env,
    sync::LazyLock,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use jsonwebtoken::{Algorithm, EncodingKey, Header, encode, errors::Error as JWTError};
use serde::{Deserialize, Serialize};

static JWT_SECRET: LazyLock<EncodingKey> = LazyLock::new(|| {
    let secret: String = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    EncodingKey::from_secret(secret.as_bytes())
});

#[derive(Serialize, Deserialize)]
pub struct JWT {
    expiration: u128,
    user_id: u32, // 16K users should be fine for now
}

impl JWT {
    pub fn create(user_id: u32, duration: Duration) -> Result<String, JWTError> {
        let now: Duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Should never fail");

        let claims: JWT = JWT {
            expiration: (now + duration).as_millis(),
            user_id,
        };

        encode(&Header::new(Algorithm::HS256), &claims, &JWT_SECRET)
    }
}
