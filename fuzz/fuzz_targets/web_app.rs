#![no_main]

use std::sync::{Mutex, OnceLock};

use axum::{
    body::Body,
    http::{header, Request},
};
use libfuzzer_sys::fuzz_target;
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use tokio::runtime::{Builder, Runtime};
use tower::ServiceExt;

#[path = "../../server/src/main.rs"]
mod server;

static APP: OnceLock<Mutex<(Runtime, axum::Router)>> = OnceLock::new();

fuzz_target!(|data: &[u8]| {
    let app = APP.get_or_init(|| {
        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("could not create fuzz runtime");
        let connection = runtime.block_on(async {
            let connection = Database::connect("sqlite::memory:")
                .await
                .expect("could not connect to in-memory database");
            Migrator::fresh(&connection)
                .await
                .expect("could not create fuzz database schema");
            Migrator::up(&connection, None)
                .await
                .expect("could not seed fuzz database");
            connection
        });
        Mutex::new((runtime, server::app(connection)))
    });
    let app = app.lock().expect("fuzz app mutex was poisoned");
    let (runtime, router) = &*app;

    let route = data.first().copied().unwrap_or_default() % 8;
    let body = data.get(1..).unwrap_or_default();

    match route {
        0 => {
            let _ = serde_json::from_slice::<dto::NewPet>(body);
        }
        1 => {
            let _ = serde_json::from_slice::<dto::Owner>(body);
        }
        2 => {
            let request = Request::builder()
                .uri("/owners")
                .body(Body::from(body.to_vec()))
                .expect("could not build owners request");
            let response = runtime.block_on(router.clone().oneshot(request)).unwrap();
            if response.status().is_success() {
                let response_body =
                    runtime.block_on(hyper::body::to_bytes(response.into_body())).unwrap();
                let _: Vec<dto::Owner> = serde_json::from_slice(&response_body)
                    .expect("owners endpoint returned an invalid client payload");
            }
        }
        3 => {
            let token_request = Request::builder()
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .uri("/token")
                .body(Body::from(body.to_vec()))
                .expect("could not build token request");
            let token_response =
                runtime.block_on(router.clone().oneshot(token_request)).unwrap();
            if token_response.status().is_success() {
                let token_body =
                    runtime.block_on(hyper::body::to_bytes(token_response.into_body())).unwrap();
                let token = String::from_utf8(token_body.to_vec())
                    .expect("token endpoint returned invalid UTF-8");
                let vets_request = Request::builder()
                    .uri("/vets")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("could not build authenticated vets request");
                let _ = runtime.block_on(router.clone().oneshot(vets_request)).unwrap();
            }
        }
        4 => {
            let request = Request::builder()
                .method("POST")
                .header(header::CONTENT_TYPE, "application/json")
                .uri("/owners/1/pets/new")
                .body(Body::from(body.to_vec()))
                .expect("could not build pet request");
            let _ = runtime.block_on(router.clone().oneshot(request)).unwrap();
        }
        5 => {
            let request = Request::builder()
                .uri("/vets")
                .body(Body::empty())
                .expect("could not build unauthenticated vets request");
            let _ = runtime.block_on(router.clone().oneshot(request)).unwrap();
        }
        6 => {
            let request = Request::builder()
                .uri("/")
                .body(Body::empty())
                .expect("could not build root request");
            let response = runtime.block_on(router.clone().oneshot(request)).unwrap();
            if let Some(location) = response.headers().get(header::LOCATION) {
                let location = location
                    .to_str()
                    .expect("root returned a non-text redirect location");
                let request = Request::builder()
                    .uri(location)
                    .body(Body::empty())
                    .expect("root returned an invalid redirect location");
                let _ = runtime.block_on(router.clone().oneshot(request)).unwrap();
            }
        }
        _ => {
            let request = Request::builder()
                .uri("/not-found")
                .body(Body::from(body.to_vec()))
                .expect("could not build unmatched-route request");
            let _ = runtime.block_on(router.clone().oneshot(request)).unwrap();
        }
    }
});
