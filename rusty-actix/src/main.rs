//! A rusty-server (a rust equivalent for json server), like the json-server has two parts
//! A web server that exposes RESTful endpoints
//! And a file reader writer that can read and manipulate a json file
//! We take the dynamic json reader approach first i.e. no struct defining a schema, just Json JsonValue
// use serde_json::Value as JsonValue;

use ::phonebook::{read_json, write_json, JsonFile, Person};
use actix_web::Result as ActixResult;
use actix_web::{error as actix_error, http::StatusCode, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
#[macro_use] // https://doc.rust-lang.org/reference/macros-by-example.html#the-macro_use-attribute
mod macros;
mod into_actix_trait;

use into_actix_trait::IntoActixResult;
use lazy_static::lazy_static;
use std::cell::RefCell;
use std::net::TcpListener;
use std::rc::Rc;
use std::sync::Once;


lazy_static! {
    static ref PHONEBOOK_PATH: &'static std::path::Path = &std::path::Path::new("files/mock.json");
}
// https://users.rust-lang.org/t/how-can-i-use-mutable-lazy-static/3751/3
static mut APP_JSON_FILE: Rc<RefCell<JsonFile>> = Rc::new(RefCell::new(JsonFile::default()));
static APP_INIT: Once = Once::new();

pub(crate) type ActixResponse = ActixResult<HttpResponse>;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init();
    env_logger::init();
    let tcp = TcpListener::bind("127.0.0.1:80")?;
    let _port = tcp.local_addr()?.port();

    HttpServer::new(move || {
        App::new()
            .route("/book", web::get().to(get_phonebook_handler))
            .route("/book", web::post().to(post_phonebook_handler))
            // .route("/book/{name}", web::get().to(get_by_name))
            // .route("/book/{id}", web::get().to(get_by_id))
    })
    .listen(tcp)?
    .run()
    .await
}

#[actix_web::get("/book/{id}")]
async fn get_by_id(req: HttpRequest, path: web::Path<phonebook::PersonID>) -> ActixResponse {
    let id = path.into_inner();

    todo!()
}

// #[actix_web::get("/book/{name}")]
// async fn get_by_name(req: HttpRequest, path: web::Path<String>) -> ActixResult<HttpResponse> {
//     let name = path.into_inner();
//     // If none found send a HTTP 204: Request was processed but no name was found
//     let result = ::phonebook::JsonFile::get_by_name(&APP_JSON_FILE, &name).unwrap_or(default)
//     todo!()
// }

async fn post_phonebook_handler(_req: HttpRequest, person: web::Json<Person>) -> ActixResponse {
    println!("POST recvd");
    let person = person.into_inner();
    println!("{person:?}");
    // SAFETY: APP_JSON_FILE is properly initialized else the app will panic at start
    let mut json_file = unsafe { &mut APP_JSON_FILE }; //read_json(path).unwrap();
    json_file.add_to_phonebook(person).map_err(|e| {
        log::warn!("{:?}", e);
        actix_error::ErrorInternalServerError(e)
    })?;
    write_json(&PHONEBOOK_PATH, &mut json_file).actix_result()?;
    Ok(HttpResponse::Ok().finish())
}

async fn get_phonebook_handler(_req: HttpRequest) -> ActixResponse {
    println!("GET recvd");
    // SAFETY: APP_JSON_FILE is properly initialized else the app will panic at start
    let json_file = unsafe { &APP_JSON_FILE }; //read_json(path).unwrap();
    // Problem serde_json::error::Result<T> is returned here and must be converted to 
    // anyhow::Result<T> before actix_result() will work
    // let payload = serde_json::to_string_pretty(&json_file).actix_result()?;
    // Fortunately, we have from actix_web
    // impl ResponseError for serde_json::Error {}

    let payload = serde_json::to_string_pretty(&json_file)?;
    Ok(HttpResponse::Ok().content_type("application/json").body(payload))
}

fn init() {
    unsafe {
        APP_INIT.call_once(|| {
            log::error!("App initialization failed");
            APP_JSON_FILE =
                read_json(&PHONEBOOK_PATH).expect("Failed to read {PHONEBOOK_PATH}. App initialization failed");
        })
    }
}
