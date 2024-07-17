use super::{action::Action, guard::OnGuard, layer::GuardActionLayer};
use axum::{routing::MethodRouter, Router};
use std::sync::Arc;

#[derive(Clone)]
pub struct GuardRouter<G, S = ()> {
    resource: String,
    roles: Option<Vec<String>>,
    actions: Vec<(String, Action<S>)>,
    guard: Arc<G>,
}

#[allow(rustdoc::invalid_rust_codeblocks)]
impl<G, S> GuardRouter<G, S>
where
    S: Clone + Send + Sync + 'static,
    G: OnGuard + Clone + Send + Sync + 'static,
{
    /// Create a guard router
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::sync::Arc;
    /// use axum_guard_router::{GuardRouter, OnGuard};
    /// use axum::{
    ///     extract::Path,
    ///     http::StatusCode,
    ///     response::{IntoResponse, Response},
    ///     routing::{get, post, put},
    ///     Json, Router,
    /// };
    ///
    /// #[derive(Clone)]
    /// struct MyGuard;
    ///
    /// impl OnGuard for MyGuard {
    ///     async fn on_guard(&self, resource: &str, action: &str) -> Result<(), Response> {
    ///         println!("on_guard: resource={resource} action={action}");
    ///         if action == "my:update" {
    ///             return Err((
    ///                    StatusCode::FORBIDDEN,
    ///                    format!("resource={resource} action={action}"),
    ///                ).into_response());
    ///          }
    ///          Ok(())
    ///        }
    ///    }
    ///
    ///  let router = GuardRouter::new("my:router:resource", Arc::new(MyGuard));
    ///
    /// ```
    pub fn new(resource: &str, guard: Arc<G>) -> Self {
        Self {
            guard,
            resource: resource.to_string(),
            actions: Vec::new(),
            roles: None,
        }
    }

    /// Create a guard router with action
    /// one path can only create one action with axum::routing::get, post, put, delete
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::sync::Arc;
    /// use axum_guard_router::{GuardRouter, OnGuard};
    /// use axum::{
    ///     extract::Path,
    ///     http::StatusCode,
    ///     response::{IntoResponse, Response},
    ///     routing::{get, post, put},
    ///     Json, Router,
    /// };
    ///
    /// #[derive(Clone)]
    /// struct MyGuard;
    ///
    /// impl OnGuard for MyGuard {
    ///     async fn on_guard(&self, resource: &str, action: &str) -> Result<(), Response> {
    ///         println!("on_guard: resource={resource} action={action}");
    ///         if action == "my:update" {
    ///             return Err((
    ///                    StatusCode::FORBIDDEN,
    ///                    format!("resource={resource} action={action}"),
    ///                ).into_response());
    ///          }
    ///          Ok(())
    ///        }
    ///    }
    ///
    ///  async fn handler1() {}
    ///  async fn handler2() {}
    ///
    ///  let router = GuardRouter::new("my:router:resource", Arc::new(MyGuard))
    ///     .action("my:create", "/user", post(handler))
    ///     .action("my:update", "/user", put(handler2));
    ///
    /// ```
    pub fn action(mut self, name: &str, path: &str, method_router: MethodRouter<S>) -> Self {
        let action = Action::create(name, method_router);
        self.actions.push((path.to_string(), action));
        self
    }

    /// Create a guard router with actions
    /// a same path can create multiple actions with action::get, post, put, delete.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::sync::Arc;
    /// use axum_guard_router::{GuardRouter, action::{post, put}, OnGuard};
    /// use axum::{
    ///     extract::Path,
    ///     http::StatusCode,
    ///     response::{IntoResponse, Response},
    ///     Json, Router,
    /// };
    ///
    /// #[derive(Clone)]
    /// struct MyGuard;
    ///
    /// impl OnGuard for MyGuard {
    ///     async fn on_guard(&self, resource: &str, action: &str) -> Result<(), Response> {
    ///         println!("on_guard: resource={resource} action={action}");
    ///         if action == "my:update" {
    ///             return Err((
    ///                    StatusCode::FORBIDDEN,
    ///                    format!("resource={resource} action={action}"),
    ///                ).into_response());
    ///          }
    ///          Ok(())
    ///        }
    ///    }
    ///
    ///  async fn handler1() {}
    ///  async fn handler2() {}
    ///
    ///  let router = GuardRouter::new("my:router:resource", Arc::new(MyGuard))
    ///     .route("/user", post("my:create", handler).put("my:update", handler2));
    ///
    /// ```
    pub fn route(mut self, path: &str, action: Action<S>) -> Self {
        self.actions.push((path.to_string(), action));
        self
    }

    /// Create a guard router with roles
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::sync::Arc;
    /// use axum_guard_router::{GuardRouter, OnGuard};
    /// use axum::{
    ///     extract::Path,
    ///     http::StatusCode,
    ///     response::{IntoResponse, Response},
    ///     routing::{get, post, put},
    ///     Json, Router,
    /// };
    ///
    /// #[derive(Clone)]
    /// struct MyGuard;
    ///
    /// impl OnGuard for MyGuard {
    ///     async fn on_roles(&self, roles: &[String]) -> Result<(), Response> {
    ///         Ok(())
    ///     }
    ///
    ///  async fn handler1() {}
    ///  async fn handler2() {}
    ///  async fn handler3() {}
    ///  async fn handler4() {}
    ///
    ///  let roles = vec!["admin".to_string()];
    ///  let router = GuardRouter::new("my:router:admin", Arc::new(MyGuard)).roles(&roles)
    ///     .action("my:create", "/admin", post(handler))
    ///     .action("my:update", "/admin", put(handler2));
    ///
    ///  let roles = vec!["user".to_string()];
    ///  let router = GuardRouter::new("my:router:user", Arc::new(MyGuard)).roles(&roles)
    ///     .action("my:create", "/user", post(handler3))
    ///     .action("my:update", "/user", put(handler4));
    ///
    /// ```
    pub fn roles(mut self, roles: &[String]) -> Self {
        self.roles = Some(roles.to_vec());
        self
    }

    /// Build guard router and generate axum router
    ///
    /// # Example
    ///
    /// ```rust,ignore
    ///  async fn handler1() {}
    ///  async fn handler2() {}
    ///  let guard_router = GuardRouter::new("my:router:admin", Arc::new(MyGuard))
    ///     .action("my:create", "/admin", post(handler))
    ///     .action("my:update", "/admin", put(handler2))
    ///     .build();
    ///
    ///  let app = Router::new().nest("/protect", guard_router);
    ///
    /// ```
    pub fn build(&self) -> Router<S> {
        let mut router = Router::<S>::new();
        for (path, action) in &self.actions {
            let mut method_router = MethodRouter::new();
            for (name, r) in action.routers() {
                method_router = method_router.merge(
                    r.layer(
                        GuardActionLayer::new(self.guard.clone(), &self.resource, &name)
                            .roles(&self.roles),
                    ),
                );
            }
            router = router.route(path, method_router);
        }
        router
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::test_helper::{TestClient, TestGuard};
    use crate::{action, router::GuardRouter};
    use axum::routing::{get, post};
    use axum::Router;
    use reqwest::StatusCode;

    #[test]
    fn test_guard_new() {
        let guid = Arc::new(TestGuard::new());
        let router = GuardRouter::<TestGuard, ()>::new("my:test", guid);
        assert_eq!(router.resource, "my:test");
    }

    #[test]
    fn test_guard_action() {
        let guid = Arc::new(TestGuard::new());
        let router = GuardRouter::<TestGuard, ()>::new("my:test", guid)
            .action("action1", "/", get(handler))
            .action("action2", "/test", post(handler2));
        assert_eq!(router.actions.len(), 2);

        assert_eq!(router.actions[0].0, "/");
        // assert_eq!(router.actions[0].1, "action1");
        assert_eq!(router.actions[1].0, "/test");
        // assert_eq!(router.actions[1].0, "action2");
    }

    #[tokio::test]
    async fn test_guard_route_forbidden() {
        let guid = Arc::new(TestGuard::new());
        let router = GuardRouter::<TestGuard, ()>::new("my:test", guid)
            .route(
                "/test",
                action::get("action1", handler).post("action2", handler2),
            )
            .build();

        let app = Router::new().nest("/api", router);
        let client = TestClient::new(app);
        let status = client.get("/api/test").await.status();
        assert_eq!(status, StatusCode::FORBIDDEN);

        let status = client.post("/api/test").await.status();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_guard_route_pass() {
        let guid = Arc::new(TestGuard::new_with(true, true));
        let router = GuardRouter::<TestGuard, ()>::new("my:test", guid)
            .route(
                "/test",
                action::get("action1", handler).post("action2", handler2),
            )
            .build();

        let app = Router::new().nest("/api", router);
        let client = TestClient::new(app);
        let status = client.get("/api/test").await.status();
        assert_eq!(status, StatusCode::OK);

        let status = client.post("/api/test").await.status();
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_guard_guard_pass() {
        let guid = Arc::new(TestGuard::new_with(true, true));
        let router = GuardRouter::<TestGuard, ()>::new("my:test", guid)
            .action("action1", "/test", get(handler))
            .build();

        let client = TestClient::new(router);
        let status = client.get("/test").await.status();
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_guard_guard_nest() {
        let guid = Arc::new(TestGuard::new_with(true, true));
        let router = GuardRouter::<TestGuard, ()>::new("my:test", guid)
            .action("action1", "/test", get(handler))
            .build();
        let app = Router::new().nest("/api", router);
        let client = TestClient::new(app);
        let status = client.get("/api/test").await.status();
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_guard_guard_on_roles_403() {
        let guid = Arc::new(TestGuard::new_with(true, false));
        let roles = vec!["admin".to_string()];
        let router = GuardRouter::<TestGuard, ()>::new("my:test", guid)
            .roles(&roles)
            .action("action1", "/test", get(handler))
            .build();

        let client = TestClient::new(router);
        let status = client.get("/test").await.status();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_guard_on_guard_403() {
        let guid = Arc::new(TestGuard::new_with(false, true));
        let router = GuardRouter::<TestGuard, ()>::new("my:test", guid)
            .action("action1", "/test", get(handler))
            .build();

        let client = TestClient::new(router);
        let status = client.get("/test").await.status();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    async fn handler() {}
    async fn handler2() {}
}
