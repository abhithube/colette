use colette_jwt::{Claims, JwtManager};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct ValidateAccessTokenQuery {
    pub access_token: String,
}

pub struct ValidateAccessTokenHandler<JM: JwtManager> {
    jwt_manager: JM,
}

impl<JM: JwtManager> ValidateAccessTokenHandler<JM> {
    pub fn new(jwt_manager: JM) -> Self {
        Self { jwt_manager }
    }
}

#[async_trait::async_trait]
impl<JM: JwtManager> Handler<ValidateAccessTokenQuery> for ValidateAccessTokenHandler<JM> {
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
