use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use axum_guard_router::{GuardRouter, OnGuard};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
struct MyGuard;

impl OnGuard for MyGuard {
    async fn on_guard(&self, resource: &str, action: &str) -> Result<(), Response> {
        println!("on_guard: resource={resource} action={action}");
        // check permission by resource and action
        if action == "my:update" {
            return Err((
                StatusCode::FORBIDDEN,
                format!("resource={resource} action={action}"),
            )
                .into_response());
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let guid = Arc::new(MyGuard);
    let app = Router::new().nest(
        "/user",
        GuardRouter::new("admin:user", guid.clone())
            .action("my:create", "/", post(create_user))
            .build()
            .nest(
                "/nest",
                GuardRouter::new("admin:user", guid.clone())
                    .action("my:get", "/:id", get(get_user))
                    .action("my:update", "/:id", put(update_user))
                    .build(),
            ),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize, Deserialize)]
struct CreateUser {
    username: String,
}

#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    username: String,
}

async fn get_user(Path(id): Path<u64>) -> impl IntoResponse {
    let user = User {
        id,
        username: "test".to_string(),
    };
    (StatusCode::OK, Json(user))
}

async fn create_user(Json(payload): Json<CreateUser>) -> impl IntoResponse {
    let user = User {
        id: 1337,
        username: payload.username,
    };
    (StatusCode::CREATED, Json(user))
}

async fn update_user(Path(id): Path<u64>, Json(mut user): Json<User>) -> impl IntoResponse {
    user.id = id;
    (StatusCode::OK, Json(user))
}
