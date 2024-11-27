use axum::{async_trait, extract::{FromRequestParts, State}, http::{request::Parts, StatusCode}, Json, RequestPartsExt};
use axum_extra::{headers::{authorization::Bearer, Authorization}, TypedHeader};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{repository::{Repository, RepositoryError}, token::{decrypt_token, encrypt_token, Claims}, user::Account, App};

#[derive(Debug, Clone, Deserialize)]
pub struct Form {
    username: String,
    password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthError {
    message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreationAccountOk {
    id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginToken {
    token: String,
}

pub async fn create_account(State(app): State<App>, Json(form): Json<Form>) -> Result<Json<CreationAccountOk>, (StatusCode, Json<AuthError>)> {
    let account = Account::new(form.username, form.password);
    match app.repository.lock().await.create_account(&account) {
        Err(err) => Err(err.into()),
        Ok(_) => Ok(Json(CreationAccountOk{id: account.id})),
    }
}

pub async fn login_account(State(app): State<App>, Json(form): Json<Form>) -> Result<Json<LoginToken>, (StatusCode, Json<AuthError>)>  {
    match app.repository.lock().await.login_account(form.username, form.password) {
        Err(err) => Err(err.into()),
        Ok(account) => {
            Ok(Json(LoginToken{token: encrypt_token(Claims::new(account.id))}))
        },
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims {
    type Rejection = (StatusCode, Json<AuthError>);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts.extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| (StatusCode::BAD_REQUEST, Json(AuthError{message: "required bearer".to_string()})))?;
        let claims = decrypt_token(bearer.token().to_string());
        match claims {
            Some(claims) => Ok(claims),
            None => Err(
                (StatusCode::BAD_REQUEST, Json(AuthError{message: "invalid token".to_string()}))
            ),
        }
    }
}

impl From<RepositoryError> for AuthError {
    fn from(value: RepositoryError) -> Self {
        match value {
            RepositoryError::NotFound => Self { message: "not found".to_string() },
            RepositoryError::DatabaseError => Self { message: "service unavaible".to_string()},
        }
    }
}

impl From<RepositoryError> for (StatusCode, Json<AuthError>) {
    fn from(value: RepositoryError) -> Self {
        (
            match value {
                RepositoryError::NotFound => StatusCode::NOT_FOUND,
                RepositoryError::DatabaseError => StatusCode::SERVICE_UNAVAILABLE,
            },
            Json(value.into()),
        )
    }
}

pub struct SelfAccount(Account);

#[async_trait]
impl FromRequestParts<App> for SelfAccount {
    type Rejection = (StatusCode, Json<AuthError>);

    async fn from_request_parts(parts: &mut Parts, app: &App) -> Result<Self, Self::Rejection> {
        let claims = parts.extract::<Claims>().await?;
        match app.repository.lock().await.get_account(claims.account_id) {
            Ok(account) => Ok(SelfAccount(account)),
            Err(err) => Err(err.into()),
        }
    }
}

pub async fn account_get_self(SelfAccount(account): SelfAccount) -> Result<Json<Account>, (StatusCode, Json<AuthError>)> {
    Ok(Json(account))
}
