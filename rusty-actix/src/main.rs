//! A rusty-server (a rust equivalent for json server), like the json-server has two parts
//! A web server that exposes RESTful endpoints
//! And a file reader writer that can read and manipulate a json file
//! We take the dynamic json reader approach first i.e. no struct defining a schema, just Json JsonValue

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::io;
// use io::BufReader;
// use io::Result;
// use io::Read;
use std::fmt::Display;
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

#[derive(Serialize, Deserialize, Debug)]
struct Person {
    id: u128,
    name: String,
    number: String,
}
impl Display for Person {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Person { name, id, number } = self;
        write!(f, "{{ name: {name} id: {id} number: {number} }})")
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonFile {
    phonebook: Vec<Person>,
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

    let mut json_file = serde_json::from_slice::<JsonFile>(&bytes)
        .map_err(|err| Err::Json(err))
        .with_context(|| "json file parse error")?;
    let mut phonebook = json_file.phonebook;

    // TODO : write procedures to manipulate the phonebook
    // For starters we want to create entries, delete entries, generate IDs using a random function
    // Then we can trouble ourselves with updating a pre-existing entry
    // also we must ensure that if a "name" already exists, it shouldn't be added with an appropriate error msg
    // "Names should be unique"

    // Let's change one entry first
    print_phonebook(&phonebook);
    mutate(&mut phonebook)?;
    println!("\nAfter Mutation\n");
    print_phonebook(&phonebook);

    Ok(())
}

fn mutate(p: &mut Vec<Person>) -> Result<()> {
    for person in p.iter_mut() {
        let p_name = &person.name;
        if p_name.to_ascii_lowercase().starts_with("arto")
            && p_name.to_ascii_lowercase().ends_with("hellas")
        {
            // *person.name is wrong syntax because `.` has higher precedence than `*`
            // (*person).name = String::from("Cassandra Fox");
            let arto = std::mem::replace(&mut person.name, String::from("Cassandra Fox"));
            println!("Arto Hellas found and Deleted! ^_^");
        }
    }

    Ok(())
}

fn print_phonebook(p: &Vec<Person>) {
    let entries = p.iter();
    for person in entries {
        println!("{person}");
    }
}
