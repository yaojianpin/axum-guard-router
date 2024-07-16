use crate::OnGuard;
use axum::{
    extract::Request,
    response::Response,
};
use futures::future::BoxFuture;
use std::{
    sync::Arc,
    task::{Context, Poll},
};
use tower::Service;

#[derive(Clone, Debug)]
pub struct GuardService<G, S> {
    pub(crate) guard: Arc<G>,
    pub(crate) inner: S,
    pub(crate) resource: String,
    pub(crate) action: String,
    pub(crate) roles: Option<Vec<String>>,
}

impl<G, S> Service<Request> for GuardService<G, S>
where
    G: OnGuard + Clone,
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        log::debug!(
            "GuardService: resource={} action={}",
            self.resource,
            self.action
        );
        let guard = self.guard.clone();

        let resource = self.resource.clone();
        let action = self.action.clone();
        let roles = self.roles.clone();
        let result = futures::executor::block_on(async move {
            if let Some(roles) = &roles {
                if let Err(ret) = guard.on_roles(roles).await {
                    return Err(ret);
                }
            }
            guard.on_guard(&resource, &action).await
        });

        if let Err(ret) = result {
            return Box::pin(async move { Ok(ret) });
        }

        let future = self.inner.call(request);
        Box::pin(async move {
            let response: Response = future.await?;
            Ok(response)
        })
    }
}
