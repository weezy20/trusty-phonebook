//! A rusty-server (a rust equivalent for json server), like the json-server has two parts
//! A web server that exposes RESTful endpoints
//! And a file reader writer that can read and manipulate a json file
//! We take the dynamic json reader approach first i.e. no struct defining a schema, just Json JsonValue

use actix_web::web::Json;
// use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use serde_json::Value as JsonValue;
use std::io;
// use io::BufReader;
// use io::Result;
// use io::Read;
use std::fs::File;
use std::path::Path;
#[derive(Debug, thiserror::Error)]
enum Err {
    #[error("IO ERROR HAPPENED!")]
    Io(#[from] io::Error),
    #[error("JSON ERROR")]
    Json(#[from] serde_json::error::Error),
    #[error("JSON Array Parse Error")]
    JsonArrayParseError,
}

fn main() -> Result<()> {
    // let path = Path::new("../phonebook.json");
    let path = Path::new("mock.json");
    let rdr = File::options()
        .write(true)
        .read(true)
        .open(path)
        .map_err(|err| Err::Io(err))
        .with_context(|| format!("Failed to read `{}`", path.display()))?;
    // The content of the IO stream is deserialized directly from the stream without being buffered in memory by serde_json.
    // let phonebook = serde_json::from_reader::<File, JsonValue>(rdr)?;
    // https://github.com/serde-rs/json/issues/160
    // https://github.com/paritytech/substrate/pull/10137
    // let buf_rdr = BufReader::new(rdr);
    // let phonebook = serde_json::from_reader::<BufReader<File>, JsonValue>(buf_rdr)?;
    // Apparently reading the entire file into memory is the fastest way to deserialize i.e. `from_slice` and `from_str` methods
    // are faster than the `from_reader` method
    let bytes = unsafe {
        memmap2::Mmap::map(&rdr)
            .map_err(|err| Err::Io(err))
            .with_context(|| "IO error at mmap")?
    };

    let mut json_file = serde_json::from_slice::<JsonValue>(&bytes)
        .map_err(|err| Err::Json(err))
        .with_context(|| "json parse error <X>")?;
    let ref mut phonebook = json_file["phonebook"];

    // TODO : write procedures to manipulate the phonebook
    // For starters we want to create entries, delete entries, generate IDs using a random function
    // Then we can trouble ourselves with updating a pre-existing entry
    // also we must ensure that if a "name" already exists, it shouldn't be added with an appropriate error msg
    // "Names should be unique"

    // Let's change one entry first
    print(phonebook);
    mutate(phonebook)?;
    println!("\nAfter Mutation\n");
    print(phonebook);

    Ok(())
}

fn mutate(p: &mut JsonValue) -> Result<()> {
    let entries = p
        .as_array_mut()
        .ok_or(Err::JsonArrayParseError)
        .with_context(|| format!("Cannot obtain a mutable array from json JsonValue"))?;
    // Cannot attach phonebook debug in this context since it is already mutably borrowed
    for entry in entries.iter_mut() {
        // It's possible to index a JsonValue by &str because of :
        // https://docs.serde.rs/serde_json/value/trait.Index.html#foreign-impls
        // println!("{}", entry["name"]);
        // Let's mutate id 1 , name from Arto Hellas to Cassandra Fox
        let arto = entry["name"]
            .as_str()
            .ok_or(Err::JsonArrayParseError)
            .with_context(|| format!("Name must be String"))?;
        if arto.to_ascii_lowercase().starts_with("arto")
            && arto.to_ascii_lowercase().ends_with("hellas")
        {
            let _ = std::mem::replace(
                &mut entry["name"],
                JsonValue::String("Cassandra Fox".into()),
            );
        }
    }

    Ok(())
}

fn print(p: &JsonValue) {
    let entries = p.as_array().expect("Infallible");
    for entry in entries.iter() {
        println!("{entry}");
    }
}
