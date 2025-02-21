use axum::body::Bytes;
use axum::http::{HeaderName, HeaderValue, Request, Response};
use axum::response::IntoResponse;
use axum::serve;
use futures::future::BoxFuture;
use reqwest::StatusCode;
use std::{convert::Infallible, future::IntoFuture, net::SocketAddr};
use tokio::net::TcpListener;
use tower::make::Shared;
use tower::Service;

use crate::OnGuard;

#[derive(Clone)]
pub struct TestGuard {
    pub guard_result: bool,
    pub roles_result: bool,
}

impl TestGuard {
    pub fn new() -> Self {
        TestGuard {
            guard_result: false,
            roles_result: false,
        }
    }

    pub fn new_with(guard_result: bool, roles_result: bool) -> Self {
        TestGuard {
            guard_result,
            roles_result,
        }
    }
}

impl OnGuard for TestGuard {
    async fn on_guard(&self, resource: &str, action: &str) -> Result<(), axum::response::Response> {
        log::debug!("on_guard: resource={resource},action={action}");
        match self.guard_result {
            true => Ok(()),
            false => Err((StatusCode::FORBIDDEN, "error").into_response()),
        }
    }

    async fn on_roles(&self, roles: &[String]) -> Result<(), axum::response::Response> {
        log::debug!("on_roles: roles={:?}", roles);
        match self.roles_result {
            true => Ok(()),
            false => Err((StatusCode::FORBIDDEN, "error").into_response()),
        }
    }
}

pub(crate) struct TestClient {
    client: reqwest::Client,
    addr: SocketAddr,
}

impl TestClient {
    pub(crate) fn new<S>(svc: S) -> Self
    where
        S: Service<
                Request<axum::body::Body>,
                Response = Response<axum::body::Body>,
                Error = Infallible,
            > + Clone
            + Send
            + 'static,
        S::Future: Send,
    {
        let addr = spawn_service(svc);

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();

        TestClient { client, addr }
    }

    pub(crate) fn get(&self, url: &str) -> RequestBuilder {
        RequestBuilder {
            builder: self.client.get(format!("http://{}{}", self.addr, url)),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn head(&self, url: &str) -> RequestBuilder {
        RequestBuilder {
            builder: self.client.head(format!("http://{}{}", self.addr, url)),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn post(&self, url: &str) -> RequestBuilder {
        RequestBuilder {
            builder: self.client.post(format!("http://{}{}", self.addr, url)),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn put(&self, url: &str) -> RequestBuilder {
        RequestBuilder {
            builder: self.client.put(format!("http://{}{}", self.addr, url)),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn patch(&self, url: &str) -> RequestBuilder {
        RequestBuilder {
            builder: self.client.patch(format!("http://{}{}", self.addr, url)),
        }
    }
}

pub(crate) struct RequestBuilder {
    builder: reqwest::RequestBuilder,
}

impl RequestBuilder {
    #[allow(dead_code)]
    pub(crate) fn body(mut self, body: impl Into<reqwest::Body>) -> Self {
        self.builder = self.builder.body(body);
        self
    }

    #[allow(dead_code)]
    pub(crate) fn json<T>(mut self, json: &T) -> Self
    where
        T: serde::Serialize,
    {
        self.builder = self.builder.json(json);
        self
    }

    #[allow(dead_code)]
    pub(crate) fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<axum::http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<axum::http::Error>,
    {
        self.builder = self.builder.header(key, value);
        self
    }

    #[allow(dead_code)]
    pub(crate) fn multipart(mut self, form: reqwest::multipart::Form) -> Self {
        self.builder = self.builder.multipart(form);
        self
    }
}

impl IntoFuture for RequestBuilder {
    type Output = TestResponse;
    type IntoFuture = BoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async {
            TestResponse {
                response: self.builder.send().await.unwrap(),
            }
        })
    }
}

#[derive(Debug)]
pub(crate) struct TestResponse {
    response: reqwest::Response,
}

impl TestResponse {
    #[allow(dead_code)]
    pub(crate) async fn bytes(self) -> Bytes {
        self.response.bytes().await.unwrap()
    }

    #[allow(dead_code)]
    pub(crate) async fn text(self) -> String {
        self.response.text().await.unwrap()
    }

    #[allow(dead_code)]
    pub(crate) async fn json<T>(self) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        self.response.json().await.unwrap()
    }

    pub(crate) fn status(&self) -> StatusCode {
        StatusCode::from_u16(self.response.status().as_u16()).unwrap()
    }

    #[allow(dead_code)]
    pub(crate) fn headers(&self) -> axum::http::HeaderMap {
        self.response.headers().clone()
    }

    #[allow(dead_code)]
    pub(crate) async fn chunk(&mut self) -> Option<Bytes> {
        self.response.chunk().await.unwrap()
    }

    #[allow(dead_code)]
    pub(crate) async fn chunk_text(&mut self) -> Option<String> {
        let chunk = self.chunk().await?;
        Some(String::from_utf8(chunk.to_vec()).unwrap())
    }
}

pub(crate) fn spawn_service<S>(svc: S) -> SocketAddr
where
    S: Service<
            Request<axum::body::Body>,
            Response = Response<axum::body::Body>,
            Error = Infallible,
        > + Clone
        + Send
        + 'static,
    S::Future: Send,
{
    let std_listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    std_listener.set_nonblocking(true).unwrap();
    let listener = TcpListener::from_std(std_listener).unwrap();

    let addr = listener.local_addr().unwrap();
    println!("Listening on {addr}");

    tokio::spawn(async move {
        serve(listener, Shared::new(svc))
            .await
            .expect("server error")
    });

    addr
}
