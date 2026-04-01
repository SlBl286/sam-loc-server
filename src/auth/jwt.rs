use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;
#[derive(Deserialize)]
pub struct Claims {
    pub user_id: u64,
}
pub fn verify_token(token: &str) -> Result<u64, String> {
    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret("secret".as_ref()),
        &Validation::default(),
    );

    match decoded {
        Ok(data) => Ok(data.claims.user_id),
        Err(_) => Err("Invalid token".to_string()),
    }
}