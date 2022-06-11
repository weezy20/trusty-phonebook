//! A rusty-server (a rust equivalent for json server), like the json-server has two parts
//! A web server that exposes RESTful endpoints
//! And a file reader writer that can read and manipulate a json file
//! We take the dynamic json reader approach first i.e. no struct defining a schema, just Json JsonValue
// use serde_json::Value as JsonValue;

#![allow(unused_imports)]
use ::phonebook::{read_json, JsonFile, Person};
use actix_cors::Cors;
use actix_files::NamedFile;
use actix_web::Result as ActixResult;
use actix_web::{error as actix_error, web, App, HttpRequest, HttpResponse, HttpServer};
use phonebook::{async_read_json, async_write_json};
#[macro_use] // https://doc.rust-lang.org/reference/macros-by-example.html#the-macro_use-attribute
mod macros;
mod into_actix_trait;
use actix_web_static_files::ResourceFiles;
use anyhow::anyhow;
use into_actix_trait::IntoActixResult;
use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::net::TcpListener;
use std::sync::{Arc, Once};
// https://users.rust-lang.org/t/how-can-i-use-mutable-lazy-static/3751/3
// Cannot call non-const fns in static/const context
lazy_static! {
    static ref PHONEBOOK_PATH: &'static std::path::Path = &std::path::Path::new("files/mock copy.json");
    static ref APP_JSON_FILE: Arc<RwLock<JsonFile>> = Arc::new(RwLock::new(JsonFile::default()));
    // Done : Select PORT from environment or start using port 80
    static ref PORT: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "80".into())
        .parse::<u16>()
        .expect("Invalid Port number");
}
static APP_INIT: Once = Once::new();
include!(concat!(env!("OUT_DIR"), "/generated.rs"));
pub(crate) type ActixResponse = ActixResult<HttpResponse>;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init();
    env_logger::init();
    std::env::set_var("RUST_LOG", "actix_web=info");
    std::env::set_var("REACT_APP_SERVER_PORT", (PORT).to_string());
    let tcp = TcpListener::bind(&format!("0.0.0.0:{}", *PORT))?;
    let _port = tcp.local_addr()?.port();
    println!("Started on port {}", *PORT);
    HttpServer::new(move || {
        let _generated = generate();
        App::new()
            // Cors::permissive is not recommended for production environments
            .wrap(Cors::permissive())
            // Get
            .route("/", web::get().to(index))
            .route("/book", web::get().to(get_phonebook_handler))
            .route("/book/{id}", web::get().to(get_by_id))
            .route("/{name}", web::get().to(get_by_name))
            // Delete
            .route("/book/{id}", web::delete().to(delete_id))
            // Post
            .route("/book", web::post().to(post_phonebook_handler))
            // Put
            // we can use "/book" and perform the checking of ids in rust or we can do better
            // and make a put "/book/id", which let's us surgically update a complete record, be it name or number
            .route("/book/{id}", web::put().to(put_update))
            // This needs to be placed after routers
            // .service(ResourceFiles::new("/", _generated))
            .service(
                actix_web_lab::web::spa()
                    .index_file("./react-front/index.html")
                    .static_resources_mount("/static")
                    .static_resources_location("./react-front/static")
                    .finish(),
            )
        // .route("/book/{name}", web::get().to(get_by_name))
    })
    .listen(tcp)?
    .run()
    .await
}

async fn index(_req: HttpRequest) -> actix_web::Result<NamedFile, std::io::Error> {
    NamedFile::open("react-front/index.html")
}

async fn put_update(path: web::Path<u32>, person: web::Json<Person>, req: HttpRequest) -> ActixResponse {
    log::info!("{} {:?} {}", req.method(), req.version(), req.uri());
    let id = path.into_inner() as ::phonebook::PersonID;
    log::info!("PUT {person:?}");
    let person = person.into_inner();
    let mutex = Arc::clone(&APP_JSON_FILE);
    let mut json_file = mutex.write();
    json_file.update(id, person).map_err(|e| {
        log::warn!("{:?}", e);
        actix_error::ErrorInternalServerError(e)
    })?;
    drop(json_file);
    // Mutex needs to be unlocked else async_write_json will fail and wait indefinitely
    async_write_json(&PHONEBOOK_PATH, Arc::clone(&mutex))
        .await
        .actix_result()?;
    Ok(HttpResponse::Ok().finish())
}

// #[actix_web::get("/book/{id}")]
async fn get_by_id(path: web::Path<u32>, req: HttpRequest) -> ActixResponse {
    log::info!("{} {:?} {}", req.method(), req.version(), req.uri());
    let id = path.into_inner() as ::phonebook::PersonID;
    let person = tokio::task::spawn_blocking(move || -> Result<Option<Person>, anyhow::Error> {
        let mutex = Arc::clone(&APP_JSON_FILE);
        let json_file = mutex.read();
        Ok(json_file.get_by_id(id))
    })
    .await
    // First we work on the JoinError
    .map_err(|_join_err| anyhow!("JoinError on get_by_id"))
    .actix_result()?
    // Then on the internal Mutex error
    // .map_err(|mutex_err| {
    //     log::error!("mutex failed at get_by_id");
    //     mutex_err
    // })
    .actix_result()?;

    if let Some(p) = person {
        let payload = serde_json::to_string_pretty(&p)?;
        Ok(HttpResponse::Ok().content_type("application/json").body(payload))
    } else {
        Ok(HttpResponse::NoContent().finish())
    }
}

// #[actix_web::get("/book/{name}")]
async fn get_by_name(req: HttpRequest, path: web::Path<String>) -> ActixResponse {
    log::info!("{} {:?} {}", req.method(), req.version(), req.uri());
    let name = path.into_inner();
    // If none found send a HTTP 204: Request was processed but no name was foun d
    let mutex = Arc::clone(&APP_JSON_FILE);
    let json_file = mutex.read();
    // .map_err(|_e| anyhow!("RwLock poisoned at function get_by_name"))
    // .actix_result()?;
    Ok(if let Some(person) = json_file.get_by_name(&name) {
        let payload = serde_json::to_string_pretty(&person)?;
        HttpResponse::Ok().content_type("application/json").body(payload)
    } else {
        HttpResponse::NoContent().finish()
    })
}

async fn post_phonebook_handler(req: HttpRequest, person: web::Json<Person>) -> ActixResponse {
    log::info!("{} {:?} {}", req.method(), req.version(), req.uri());
    log::info!("POST {person:?}");
    let person = person.into_inner();
    // SAFETY: APP_JSON_FILE is properly initialized else the app will panic at start
    let mutex = Arc::clone(&APP_JSON_FILE);
    // Create a scope for mutex guard
    // If the Mutex was "poisoned" we should just `expect` on it since the poison happened on some other thread
    // that we don't control. Should return internal server error
    let mut json_file = mutex.write();
    json_file.add_to_phonebook(person).map_err(|e| {
        log::warn!("{:?}", e);
        actix_error::ErrorInternalServerError(e)
    })?;
    drop(json_file);
    // Mutex needs to be unlocked else async_write_json will fail and wait indefinitely
    async_write_json(&PHONEBOOK_PATH, Arc::clone(&mutex))
        .await
        .actix_result()?;
    Ok(HttpResponse::Ok().finish())
}

async fn get_phonebook_handler(req: HttpRequest) -> ActixResponse {
    log::info!("{} {:?} {}", req.method(), req.version(), req.uri());
    // SAFETY: APP_JSON_FILE is properly initialized else the app will panic at start
    let json_file = APP_JSON_FILE.read();

    // Problem serde_json::error::Result<T> is returned here and must be converted to
    // anyhow::Result<T> before actix_result() will work
    // let payload = serde_json::to_string_pretty(&json_file).actix_result()?;
    // Fortunately, we have from actix_web
    // impl ResponseError for serde_json::Error {}

    let payload = serde_json::to_string_pretty(&*json_file)?;
    Ok(HttpResponse::Ok().content_type("application/json").body(payload))
}

async fn delete_id(req: HttpRequest, id: web::Path<u32>) -> ActixResponse {
    log::info!("{} {:?} {}", req.method(), req.version(), req.uri());
    let id = id.into_inner() as ::phonebook::PersonID;
    let json_file = Arc::clone(&APP_JSON_FILE);
    // Infallible
    json_file.write().delete(id).expect("Infallible");
    async_write_json(&PHONEBOOK_PATH, json_file).await.actix_result()?;

    Ok(HttpResponse::NoContent().finish())
}

fn init() {
    APP_INIT.call_once(|| {
        // TODO: Async read_json inside call_once || Not required since this is the app start anyway
        let json_file = read_json(&PHONEBOOK_PATH).expect("Failed to read {PHONEBOOK_PATH}. App initialization failed");
        let mut mutex = APP_JSON_FILE.write(); //.expect("Infallible");
        mutex.sort();
        *mutex = json_file;
    })
}
