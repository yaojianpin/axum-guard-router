use axum::response::Response;

#[allow(async_fn_in_trait)]
pub trait OnGuard {
    async fn on_guard(&self, _resource: &str, _action: &str) -> Result<(), Response> {
        Ok(())
    }
    async fn on_roles(&self, _roles: &[String]) -> Result<(), Response> {
        Ok(())
    }
}
