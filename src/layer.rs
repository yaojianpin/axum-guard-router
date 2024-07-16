use std::sync::Arc;

use super::service::GuardService;
use crate::guard::OnGuard;
use tower::Layer;

#[derive(Clone, Debug)]
pub struct GuardActionLayer<G> {
    pub guard: Arc<G>,
    pub resource: String,
    pub action: String,
    pub roles: Option<Vec<String>>,
}

impl<G> GuardActionLayer<G>
where
    G: OnGuard,
{
    pub fn new(guard: Arc<G>, resource: &str, action: &str) -> Self {
        Self {
            guard,
            resource: resource.to_string(),
            action: action.to_string(),
            roles: None,
        }
    }

    pub fn roles(mut self, roles: &Option<Vec<String>>) -> Self {
        self.roles.clone_from(roles);
        self
    }
}

impl<G, S> Layer<S> for GuardActionLayer<G>
where
    G: OnGuard + Clone,
{
    type Service = GuardService<G, S>;

    fn layer(&self, inner: S) -> Self::Service {
        GuardService {
            guard: self.guard.clone(),
            inner,
            resource: self.resource.clone(),
            action: self.action.clone(),
            roles: self.roles.clone(),
        }
    }
}
