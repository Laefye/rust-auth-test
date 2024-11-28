use std::sync::Arc;

use axum::{async_trait, extract::{FromRequestParts, Query, State}, http::{request::Parts, StatusCode}, response::IntoResponse, routing::{get, post}, Json, RequestPartsExt, Router};
use axum_extra::{headers::{authorization::Bearer, Authorization}, TypedHeader};
use network::{database::{DatabaseError, PoloDB}, post::{self, PostAccess, PostInfo}, user::{Me, UserError}, Network};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

mod network;

#[derive(Clone)]
struct AppState {
    network: Arc<Mutex<Network>>,
}

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new()
        .route("/user/create", post(create_user))
        .route("/user/login", post(login_user))
        .route("/user/me", get(get_me))
        .route("/post", post(post_post))
        .route("/post/me", get(my_posts))
        .with_state(AppState {
            network: Arc::new(Mutex::new(Network::new(PoloDB::new())))
        });

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[derive(Clone, Serialize)]
struct RestError {
    message: String,
    #[serde(skip_serializing)]
    status: StatusCode,
}

impl IntoResponse for RestError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status;
        let mut json = Json(self).into_response();
        *(json.status_mut()) = status;
        json
    }
}

impl RestError {
    pub fn new<T: ToString>(message: T, status: StatusCode) -> Self {
        Self {
            message: message.to_string(),
            status,
        }
    }
}

impl From<DatabaseError> for RestError {
    fn from(value: DatabaseError) -> Self {
        match value {
            DatabaseError::UnknownError => Self::new("database is unavaible", StatusCode::SERVICE_UNAVAILABLE),
        }
    }
}

impl From<UserError> for RestError {
    fn from(value: UserError) -> Self {
        match value {
            UserError::DatabaseError(database_error) => database_error.into(),
            UserError::UserExists => Self::new("user exists", StatusCode::CONFLICT),
            UserError::InvalidCreditinals => Self::new("invalid creditinals", StatusCode::UNAUTHORIZED),
            UserError::InvalidToken => Self::new("invalid token", StatusCode::UNAUTHORIZED),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct UserForm {
    username: String,
    password: String,
}

async fn create_user(State(app): State<AppState>, Json(form): Json<UserForm>) -> Result<Json<Uuid>, RestError> {
    app.network.lock()
        .await.user_manager()
        .create_user(form.username, form.password)
        .map(|id| Json(id))
        .map_err(|x| x.into())
}

#[derive(Debug, Clone, Serialize)]
struct LoginToken {
    token: String,
}

async fn login_user(State(app): State<AppState>, Json(form): Json<UserForm>) -> Result<Json<LoginToken>, RestError> {
    app.network.lock()
        .await.user_manager()
        .login(form.username, form.password)
        .map(|token| Json(LoginToken{ token }))
        .map_err(|x| x.into())
}

#[derive(Debug, Clone)]
struct HeaderToken(String);

#[async_trait]
impl<S> FromRequestParts<S> for HeaderToken
where
    S: Send + Sync,
{
    type Rejection = RestError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| RestError::from(UserError::InvalidToken))?;
        Ok(Self(bearer.token().to_string()))
    }
}

async fn get_me(State(app): State<AppState>, HeaderToken(token): HeaderToken) -> Result<Json<Me>, RestError> {
    let network = app.network.lock().await;
    let user = network.user_manager().get_user_access(token)?;
    Ok(Json(user.get_me()))
}

#[derive(Debug, Clone, Deserialize)]
struct PostForm {
    text: String,
}

async fn post_post(State(app): State<AppState>, HeaderToken(token): HeaderToken, Json(form): Json<PostForm>) -> Result<Json<Uuid>, RestError> {
    let network = app.network.lock().await;
    let mut user = network.user_manager().get_user_access(token)?;
    let post = user.post(form.text)?;
    Ok(Json(post.id()))
}

#[derive(Debug, Clone, Deserialize)]
struct PostsQuery {
    offset: Option<usize>,
    limit: Option<usize>,
}

async fn my_posts(State(app): State<AppState>, HeaderToken(token): HeaderToken, Query(query): Query<PostsQuery>) -> Result<Json<Vec<PostInfo>>, RestError> {
    let network = app.network.lock().await;
    let user = network.user_manager().get_user_access(token)?;
    let posts = user.get_my_posts(
        query.offset.unwrap_or(0),
        query.limit.unwrap_or(30),
    )?;
    Ok(Json(posts.iter().map(PostAccess::info).collect()))
}
