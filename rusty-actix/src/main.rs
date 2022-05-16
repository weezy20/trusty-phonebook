//! A rusty-server (a rust equivalent for json server), like the json-server has two parts
//! A web server that exposes RESTful endpoints
//! And a file reader writer that can read and manipulate a json file
//! We take the dynamic json reader approach first i.e. no struct defining a schema, just Json JsonValue

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
// use serde_json::Value as JsonValue;
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
    #[error("Phonebook entry doesn't match expectation")]
    PhonebookEntry,
}

type PersonID = u128;

#[derive(Serialize, Deserialize, Debug, Default)]
struct Person {
    id: PersonID,
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
    env_logger::init();
    // Test adding to an empty json
    // let path = Path::new("files/mock_empty.json");
    // let out_path = Path::new("files/mock_out_empty.json");
    // Test reading and writing to same file: 
    
    let path = Path::new("files/mock.json");
    let mut json_file = read_json(&path)?;

    // TODO : write procedures to manipulate the phonebook
    // Then we can trouble ourselves with updating a pre-existing entry
    // also we must ensure that if a "name" already exists, it shouldn't be added with an appropriate error msg
    // "Names should be unique"

    // TODO : Write a convenient decl macro for add_to_phonebook which removes the need for ..Default::default()
    json_file.print_phonebook();
    json_file.add_to_phonebook(
        Person {
            name: "Abhishek R Shah".into(),
            number: "999-123".into(),
            ..Default::default()
        },
    )?;
    // This should be rejected because name isn't unique, only the whitespaces are more
    json_file.add_to_phonebook(
        Person {
            name: "Abhishek   R     Shah".into(),
            number: "999-123".into(),
            ..Default::default()
        }
    )?;
    json_file.add_to_phonebook(
        Person {
            name: "Harry Potter".into(),
            number: "4413".into(),
            ..Default::default()
        }
    )?;
    log::debug!("\nAfter Mutation:\n");
    json_file.print_phonebook();
    // Write updated phonebook to file :
    write_json(&path, &json_file)?;

    Ok(())
}

fn write_json(path: &Path, json_file: &JsonFile) -> Result<()> {
    // You can use this lib for locking https://docs.rs/fs2/latest/fs2/trait.FileExt.html
    // and this for ensuring everything got written https://doc.rust-lang.org/std/io/trait.Write.html#method.write_all
    let wrt = File::options().write(true).open(path).map_err(|err| Err::Io(err)).with_context(|| format!("Writing json failed at `{}`", path.display()))?;
    serde_json::to_writer_pretty(wrt, &json_file)?;
    Ok(())
}

fn read_json(path: &Path) -> Result<JsonFile> {
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

    serde_json::from_slice::<JsonFile>(&bytes)
        .map_err(|err| Err::Json(err))
        .with_context(|| "json file parse error")    
}
#[allow(unused)]
impl JsonFile {

/// Delete an entry 
pub fn delete(&mut self, id: PersonID) -> Result<()> {
    // iter() returns references
    // self.phonebook = self.phonebook.into_iter().filter(|p| p.id != id).collect();
    let before = self.phonebook.len();
    self.phonebook.retain(|p| p.id != id);
    let after = self.phonebook.len();
    if after - before == 0 {
        log::info!("DELETE: id #{id} doesn't exist");
    }
    Ok(())
}
/// Edit a pre-existing phonebook entry
pub fn update(&mut self, id: PersonID, p: Person) -> Result<()> {
    let entry = self
        .phonebook
        .iter_mut()
        .find(|person| person.id == id)
        .ok_or(Err::PhonebookEntry)
        .with_context(|| {
            log::info!("id: {id} does not exist in the phonebook");
            "id does not exist in phonebook"
        })?;
    
    let (nname, nnum) = (p.name, p.number);
    // Check if they are not default values
    if nname.len() > 0 {
        entry.name = nname;
    }
    if nnum.len() > 0 {
        entry.number = nnum;
    }
    // Ignore ID change requests
    Ok(())
}
// TODO : Sort by key (id) and then perform a binary search for performance gains
/// Fetch a person details by their id
pub fn get(&self, id: PersonID) -> Option<&Person> {
    self.phonebook.iter().find(|p| p.id == id)
}

pub fn print_phonebook(&self) {
    let entries = self.phonebook.iter();
    for person in entries {
        println!("{person}");
    }
}

/// Add to a phonebook only if that name is unique
pub fn add_to_phonebook(&mut self, mut p: Person) -> Result<()> {
    // Handle bad requests such as an `id` not being in their default state 0_u128
    if self.get(p.id).is_some() {
        log::warn!("Person with id {} already exists in the phonebook",p.id);
        return Err(Err::PhonebookEntry).with_context(|| format!("Person with id {} already exists, please do not provide an id", p.id));
    }
    let id = self.generate_id();
    p.id = id;
    if !self.check_if_name_exists(&p.name)? {
        self.phonebook.push(p);
    } else {
        log::warn!(
            "Name {} already exists in the phonebook. Names must be unique",
            &p.name
        );
    }
    Ok(())
}

fn generate_id(&self) -> PersonID {
    let max_phonebook_id = self
        .phonebook
        .iter()
        .max_by_key(|person| (**person).id)
        .and_then(|person| Some(person.id))
        .unwrap_or(<PersonID>::default());
    /* IDs should start with 1 incase this phonebook is empty */
    // Generates a very large id
    let mut candidate = if max_phonebook_id == 0 {
        1
    } else {
        max_phonebook_id + 1
    };
    while matches!(self.phonebook.iter().next(), Some(person) if person.id == candidate ) {
        // This debug should practically never log
        log::debug!("candidate ID collision found");
        candidate = rand::random::<PersonID>();
    }
    candidate
}

fn check_if_name_exists(&self, new_name: &str) -> Result<bool> {
    let new_name = new_name.trim().to_lowercase();
    let mut new_name = new_name.split_whitespace();
    // Important to get length before calling any `next`|`next_back`
    let new_name_len = new_name.size_hint();
    let (new_fname, new_lname) = (
        new_name
            .next()
            .ok_or(Err::PhonebookEntry)
            .with_context(|| "Phonebook entry should have a first name")?,
        new_name.next_back(),
    );
    // We take the size_hint before calling `next` and `next_back` on new_name and name
    // exactly once
    Ok(self
        .phonebook
        .iter()
        .map(|person| person.name.trim().to_lowercase())
        .any(|pname| {
            let mut name = pname.split_whitespace();
            let name_len = name.size_hint();
            // compare upper bounds
            name_len.1 == new_name_len.1 
                && name.next().expect("Safe to unwrap pre-existing fname") == new_fname
                && matches!(name.next_back(), Some(lname) if matches!(new_lname, Some(n) if n == lname))
        }))
    }
}
