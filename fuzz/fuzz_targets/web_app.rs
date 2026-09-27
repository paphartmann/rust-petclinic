#![no_main]

use std::cell::RefCell;

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

thread_local! {
    static APP: RefCell<Option<(Runtime, axum::Router)>> = const { RefCell::new(None) };
}

fuzz_target!(|data: &[u8]| {
    APP.with(|app| {
        let mut app = app.borrow_mut();
        if app.is_none() {
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
            *app = Some((runtime, server::app(connection)));
        }

        let (runtime, router) = app.as_ref().expect("fuzz app was not initialized");
        let route = match data.first() {
            Some(b'0') => 0,
            Some(b'1') => 1,
            Some(b'2') => 2,
            Some(b'3') => 3,
            Some(b'4') => 4,
            Some(byte) => byte % 5,
            None => 0,
        };
        let body = data.get(1..).unwrap_or_default();

        match route {
            0 => {
                let request = Request::builder()
                    .uri("/owners")
                    .body(Body::from(body.to_vec()))
                    .expect("could not build owners request");
                let response = runtime.block_on(router.clone().oneshot(request)).unwrap();
                if response.status().is_success() {
                    let response_body =
                        runtime.block_on(hyper::body::to_bytes(response.into_body())).unwrap();
                    let _: Result<Vec<dto::Owner>, _> = serde_json::from_slice(&response_body);
                }
            }
            1 => {
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
                        .expect("could not build vets request");
                    let _ = runtime.block_on(router.clone().oneshot(vets_request)).unwrap();
                }
            }
            2 => {
                let request = Request::builder()
                    .method("POST")
                    .header(header::CONTENT_TYPE, "application/json")
                    .uri("/owners/1/pets/new")
                    .body(Body::from(body.to_vec()))
                    .expect("could not build pet request");
                let _ = runtime.block_on(router.clone().oneshot(request)).unwrap();
            }
            3 => {
                let request = Request::builder()
                    .uri("/vets")
                    .body(Body::empty())
                    .expect("could not build vets request");
                let _ = runtime.block_on(router.clone().oneshot(request)).unwrap();
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
});
