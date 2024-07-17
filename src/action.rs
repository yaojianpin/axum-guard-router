use axum::{
    handler::Handler,
    routing::{MethodFilter, MethodRouter},
};
use std::{convert::Infallible, vec};

macro_rules! top_level_acion_fn {
    (
        $name:ident, GET
    ) => {
        top_level_acion_fn!(
            /// Route `GET` requests to the given action.
            $name,
            GET
        );
    };

    (
        $name:ident, $method:ident
    ) => {
        top_level_acion_fn!(
            #[doc = concat!("Route `", stringify!($method) ,"` requests to the given handler.")]
            $name,
            $method
        );
    };

    (
        $(#[$m:meta])+
        $name:ident, $method:ident
    ) => {
        $(#[$m])+
        pub fn $name<H, T, S>(name:&str, handler: H) -> Action<S>
        where
            H: Handler<T, S>,
            T: 'static,
            S: Clone + Send + Sync + 'static,
        {
            on(MethodFilter::$method, name, handler)
        }
    };
}

macro_rules! chained_handler_fn {
    (
        $name:ident, GET
    ) => {
        chained_handler_fn!(
            /// Route `GET` requests to the given action.
            $name,
            GET
        );
    };
    (
        $name:ident, $method:ident
    ) => {
        chained_handler_fn!(
            #[doc = concat!("Chain an additional handler that will only accept `", stringify!($method),"` requests.")]
            $name,
            $method
        );
    };

    (
        $(#[$m:meta])+
        $name:ident, $method:ident
    ) => {
        $(#[$m])+
        #[track_caller]
        pub fn $name<H, T>(self, name: &str, handler: H) -> Self
        where
            H: Handler<T, S>,
            T: 'static,
            S: Send + Sync + 'static,
        {
            self.on(MethodFilter::$method, name, handler)
        }
    };
}
/// create an action router with action name
/// ```rust, ignore
///  use axum_guard_router::{action, GuardRouter};
///  let router = GuardRouter::new("my:router:resource", Arc::new(MyGuard))
///     .route("/user", action::post("my:create", handler).put("my:update", handler2));
/// ```
#[must_use]
#[derive(Clone)]
pub struct Action<S = (), E = Infallible> {
    routers: Vec<(String, MethodRouter<S, E>)>,
}

impl<S> Action<S, Infallible>
where
    S: Clone,
{
    pub fn new() -> Self {
        Self { routers: vec![] }
    }

    #[track_caller]
    pub(crate) fn on<H, T>(mut self, filter: MethodFilter, name: &str, handler: H) -> Self
    where
        H: Handler<T, S>,
        T: 'static,
        S: Send + Sync + 'static,
    {
        self.routers
            .push((name.to_string(), MethodRouter::new().on(filter, handler)));
        self
    }

    pub(crate) fn routers(&self) -> Vec<(String, MethodRouter<S>)> {
        self.routers.clone()
    }

    pub(crate) fn create(name: &str, method_router: MethodRouter<S>) -> Self {
        Self {
            routers: vec![(name.to_string(), method_router)],
        }
    }

    chained_handler_fn!(delete, DELETE);
    chained_handler_fn!(get, GET);
    chained_handler_fn!(head, HEAD);
    chained_handler_fn!(options, OPTIONS);
    chained_handler_fn!(patch, PATCH);
    chained_handler_fn!(post, POST);
    chained_handler_fn!(put, PUT);
    chained_handler_fn!(trace, TRACE);
}

top_level_acion_fn!(delete, DELETE);
top_level_acion_fn!(get, GET);
top_level_acion_fn!(head, HEAD);
top_level_acion_fn!(options, OPTIONS);
top_level_acion_fn!(patch, PATCH);
top_level_acion_fn!(post, POST);
top_level_acion_fn!(put, PUT);
top_level_acion_fn!(trace, TRACE);

fn on<H, T, S>(filter: MethodFilter, name: &str, handler: H) -> Action<S>
where
    H: Handler<T, S>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    Action::new().on(filter, name, handler)
}
