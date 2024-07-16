use axum::response::Response;

/// A guard trati to run before a handler process
///
#[allow(async_fn_in_trait)]
pub trait OnGuard {
    /// Check the handler with resource and action
    ///  If it is not allowed, return error response
    async fn on_guard(&self, _resource: &str, _action: &str) -> Result<(), Response> {
        Ok(())
    }

    /// Check the handler with given roles
    /// If it is not allowed, return error response
    async fn on_roles(&self, _roles: &[String]) -> Result<(), Response> {
        Ok(())
    }
}
