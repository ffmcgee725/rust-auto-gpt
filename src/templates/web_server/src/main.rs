use actix_cors::Cors;
use actix_web::{http::header, web, App, HttpResponse, HttpServer, Responder};
use async_trait::async_trait;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CatImage {
    url: String,
}

struct AppState {
    client: HttpClient,
}

async fn fetch_random_cat_image(app_state: web::Data<AppState>) -> impl Responder {
    let response = app_state.client.get("https://api.thecatapi.com/v1/images/search").send().await;

    match response {
        Ok(mut resp) => {
            if resp.status().is_success() {
                let cat_images: Vec<CatImage> = resp.json().await.unwrap();
                HttpResponse::Ok().json(cat_images[0].clone())
            } else {
                HttpResponse::InternalServerError().finish()
            }
        }
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = HttpClient::new();

    let data: web::Data<AppState> = web::Data::new(AppState { client });

    return HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::permissive()
                    .allowed_origin_fn(|origin, _req_head| {
                        origin.as_bytes().starts_with(b"http://localhost") || origin == "null"
                    })
                    .allowed_methods(vec!["GET"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
            .app_data(data.clone())
            .route("/cat", web::get().to(fetch_random_cat_image))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await;
}