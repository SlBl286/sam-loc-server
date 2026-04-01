use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i64,
    pub exp: usize,
}

pub fn create_token(user_id: i64) -> String {
    let claims = Claims {
        user_id,
        exp: 2000000000,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("secret".as_ref()),
    )
    .unwrap()
}