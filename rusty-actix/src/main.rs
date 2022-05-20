//! A rusty-server (a rust equivalent for json server), like the json-server has two parts
//! A web server that exposes RESTful endpoints
//! And a file reader writer that can read and manipulate a json file
//! We take the dynamic json reader approach first i.e. no struct defining a schema, just Json JsonValue
// use serde_json::Value as JsonValue;

use ::phonebook::{read_json, write_json, Person};
use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use anyhow::Result;
#[macro_use] // https://doc.rust-lang.org/reference/macros-by-example.html#the-macro_use-attribute
mod macros;
use std::{net::TcpListener, path::Path};
// use io::BufReader;
// use io::Result;
// use io::Read;

static mut APP_JSON_FILE: Option<phonebook::JsonFile> = None;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let tcp = TcpListener::bind("127.0.0.1:80")?;
    let _port = tcp.local_addr()?.port();
    // let json_file = app_init();
    // Test adding to an empty json
    // let path = Path::new("files/mock_empty.json");
    // let out_path = Path::new("files/mock_out_empty.json");
    // Test reading and writing to same file:
    HttpServer::new(move || {
        App::new()
            .route("/book", web::get().to(get_phonebook_handler))
            .route("/book", web::post().to(post_phonebook_handler))
    })
    .listen(tcp)?
    .run()
    .await
}

async fn post_phonebook_handler(_req: HttpRequest, person: web::Json<Person>) -> impl Responder {
    println!("POST recvd");
    let path = Path::new("files/mock.json");
    let person = person.into_inner();
    println!("{person:?}");
    unsafe {
        if let Some(ref mut json_file) = APP_JSON_FILE {
            json_file.add_to_phonebook(person).unwrap();
            write_json(path, json_file).unwrap();
        } else {
            let mut json_file = read_json(path).ok().unwrap();
            json_file.add_to_phonebook(person).unwrap();

            write_json(path, &mut json_file).unwrap();
            APP_JSON_FILE = Some(json_file);
        }
    }
    HttpResponse::Ok().finish()
}

async fn get_phonebook_handler(_req: HttpRequest) -> impl Responder {
    println!("GET recvd");
    let json_file = read_json(Path::new("files/mock.json")).unwrap();
    let payload = serde_json::to_string_pretty(&json_file).unwrap();
    unsafe {
        APP_JSON_FILE = Some(json_file);
    }
    HttpResponse::Ok()
        .content_type("application/json")
        .body(payload)
}

#[test]
fn test_methods() -> Result<()> {
    let path = Path::new("files/mock.json");
    let mut json_file = read_json(&path)?;

    println!("Before any operation:");
    json_file.print_phonebook();
    json_file.add_to_phonebook(person!("Abhishek R Shah", "999-123"))?;
    // This should be rejected because name isn't unique, only the whitespaces are more
    json_file.add_to_phonebook(person!("Abhishek   R     Shah", "999-123"))?;
    json_file.add_to_phonebook(person!("Harry puttar", "999-123123128930yu1893h"))?;
    json_file.update(1, person!("Cassandra Fox", "099-887766"))?;
    json_file.delete(4)?;
    log::debug!("\nAfter Mutation:\n");
    json_file.print_phonebook();
    println!("Writing JSON to {}", path.display());
    // Write updated phonebook to file :
    write_json(&path, &mut json_file)?;

    debug_assert_eq!(None, json_file.get(10));
    Ok(())
}
