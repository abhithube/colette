use colette_jwt::{Claims, JwtManager};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct ValidateAccessTokenQuery {
    pub access_token: String,
}

pub struct ValidateAccessTokenHandler {
    jwt_manager: Box<dyn JwtManager>,
}

impl ValidateAccessTokenHandler {
    pub fn new(jwt_manager: impl JwtManager) -> Self {
        Self {
            jwt_manager: Box::new(jwt_manager),
        }
    }
}

#[async_trait::async_trait]
impl Handler<ValidateAccessTokenQuery> for ValidateAccessTokenHandler {
    type Response = Claims;
    type Error = ValidateAccessTokenError;

    async fn handle(&self, query: ValidateAccessTokenQuery) -> Result<Self::Response, Self::Error> {
        let claims = self.jwt_manager.verify(&query.access_token)?;

        Ok(claims)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidateAccessTokenError {
    #[error(transparent)]
    Jwt(#[from] colette_jwt::Error),
}
