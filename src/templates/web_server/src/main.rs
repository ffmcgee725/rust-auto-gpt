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
struct Token {
    address: String,
    block: Option<u64>,
    price: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Database {
    tokens: HashMap<String, Token>,
}

impl Database {
    fn new() -> Self {
        Self {
            tokens: HashMap::new(),
        }
    }

    fn insert(&mut self, token: Token) {
        self.tokens.insert(token.address.clone(), token);
    }

    fn get(&self, address: &str, block: Option<u64>) -> Option<&Token> {
        self.tokens.get(address)
    }

    fn save_to_file(&self) -> std::io::Result<()> {
        let data: String = serde_json::to_string(&self)?;
        let mut file: fs::File = fs::File::create("database.json")?;
        file.write_all(data.as_bytes())?;
        Ok(())
    }

    fn load_from_file() -> std::io::Result<Self> {
        let file_content: String = fs::read_to_string("database.json")?;
        let db: Database = serde_json::from_str(&file_content)?;
        Ok(db)
    }
}

struct AppState {
    db: Mutex<Database>,
    http_client: HttpClient,
}

async fn get_token_price(
    app_state: web::Data<AppState>,
    token: web::Json<Token>,
) -> impl Responder {
    let mut db = app_state.db.lock().unwrap();
    let token_in_db = db.get(&token.address, token.block);

    match token_in_db {
        Some(token) => HttpResponse::Ok().json(token),
        None => {
            let price =
                fetch_price_from_etherscan(&app_state.http_client, &token.address, token.block)
                    .await;
            match price {
                Ok(price) => {
                    let new_token = Token {
                        address: token.address.clone(),
                        block: token.block,
                        price,
                    };
                    db.insert(new_token.clone());
                    let _ = db.save_to_file();
                    HttpResponse::Ok().json(new_token)
                }
                Err(_) => HttpResponse::InternalServerError().finish(),
            }
        }
    }
}

async fn fetch_price_from_etherscan(
    client: &HttpClient,
    address: &str,
    block: Option<u64>,
) -> Result<f64, ()> {
    //     Agent: Solutions Architect: Testing URL Endpoint: https://api.etherscan.io/api?module=account&action=tokenbalance&contractaddress={token_address}&address={client_address}&tag={block_number}&apikey=YourApiKeyToken
    // Agent: Solutions Architect: Testing URL Endpoint: https://api.etherscan.io/api?module=proxy&action=eth_getBlockByNumber&tag={block_number}&boolean=true&apikey=YourApiKeyToken
    // Fetch price from etherscan API
    // This is a placeholder, replace with actual API call
    Ok(0.0)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = match Database::load_from_file() {
        Ok(db) => db,
        Err(_) => Database::new(),
    };

    let data: web::Data<AppState> = web::Data::new(AppState {
        db: Mutex::new(db),
        http_client: HttpClient::new(),
    });

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::permissive()
                    .allowed_origin_fn(|origin, _req_head| {
                        origin.as_bytes().starts_with(b"http://localhost") || origin == "null"
                    })
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
            .app_data(data.clone())
            .route("/token", web::post().to(get_token_price))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
