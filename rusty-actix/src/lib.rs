use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs::File;
use std::io;
use std::path::Path;
use std::sync::Arc;
// use std::sync::RwLock;
use parking_lot::RwLock;
#[macro_use]
mod macros;
// If interested in the gory details of anyhow::Result<T, E = anyhow::Error>
// https://rust-lang.github.io/rfcs/0213-defaulted-type-params.html
// https://rust-lang.github.io/rfcs/0213-defaulted-type-params.html#type-parameters-with-defaults
// TL;DR : Optional type params must after all non-optional ones

#[derive(Debug, thiserror::Error)]
pub enum Err {
    #[error("IO ERROR HAPPENED!")]
    Io(#[from] io::Error),
    #[error("JSON ERROR")]
    Json(#[from] serde_json::error::Error),
    #[error("Phonebook entry doesn't match expectation")]
    PhonebookEntry(String),
}

// impl actix_web::error::ResponseError for Err {}

pub type PersonID = u128;
// TODO : How is PartialEq and PartialOrd implemented for Person struct?
#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct Person {
    #[serde(default)]
    pub id: PersonID,
    pub name: String,
    pub number: String,
}
impl Display for Person {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Person { name, id, number } = self;
        write!(f, "{{ name: {name} id: {id} number: {number} }})")
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct JsonFile {
    phonebook: Vec<Person>,
}

// An alternative to JsonFile

#[derive(Serialize, Deserialize, Debug, Clone)]
// #[serde(deny_unknown_fields)] // panics
pub struct JsonFile2 {
    pub phonebook: Phonebook,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(from = "Vec<Person>", into = "Vec<Person>")]
// from = Vec<Person> means here, deserialize this type i.e Phonebook into Vec<Person> then convert it
// using a from impl to Phonebook(Hashmap)
pub struct Phonebook(pub HashMap<PersonID, Person>);

// For deserializing
impl From<Vec<Person>> for Phonebook {
    fn from(persons: Vec<Person>) -> Self {
        // TODO : Does this from fail when the json contains malformed entries, like a missing id?
        let map = persons.into_iter().map(|p| (p.id, p)).collect();
        Self(map)
    }
}
// For serializing
impl From<Phonebook> for Vec<Person> {
    fn from(pb: Phonebook) -> Self {
        // Clone required because of this
        // because it needs to clone it to get an owned copy to convert into a vec
        pb.0.into_values().collect::<Vec<Person>>()
    }
}
/// Read a JsonFile and write it to a Path, while locking the file
pub fn write_json(path: &Path, json_file: &JsonFile) -> Result<()> {
    // You can use this lib for locking https://docs.rs/fs2/latest/fs2/trait.FileExt.html
    // and this for ensuring everything got written https://doc.rust-lang.org/std/io/trait.Write.html#method.write_all
    let wrt = File::options()
        .write(true)
        // Truncate will delete the file if it exists which may not be optimal 
        // We can instead write to a temporary file, then flush the temporary file
        // to the original file path
        // In our case, since the file contents live in program memory, it makes 
        // no difference to the end user.
        // Truncation is especially helpful when delete is invoked, as post requests 
        // always add data and therefore don't need truncation
        .truncate(true) 
        .open(path)
        .map_err(|err| Err::Io(err))
        .with_context(|| format!("Writing json failed at `{}`", path.display()))?;
    // This library provides whole-file locks in both shared (read) and exclusive (read-write) varieties.
    // Uncomment this line to see the file being deleted in the interim
    // std::thread::sleep(std::time::Duration::from_secs(40));
    use fs2::FileExt;
    wrt.try_lock_exclusive()
        .map_err(|err| Err::Io(err))
        .with_context(|| "Error on locking file")?;
    // https://stackoverflow.com/questions/57232515/why-does-serde-jsonto-writer-not-require-its-argument-to-be-mut
    // https://doc.rust-lang.org/std/io/trait.Write.html#implementors
    // io::Write takes a &mut &File here
    // the mutablilty of a binding and the mutability of the bound value are not necessarily the same.
    // json_file.sort();
    serde_json::to_writer_pretty(&wrt, &json_file)?;
    wrt.unlock()
        .map_err(|err| Err::Io(err))
        .with_context(|| "Error on unlocking locking file")?;
    Ok(())
}
// 'static lifetime is fine here since our JsonFile handle and path will remain the same for the entirety of the program
// however, TOOD: explore alternatives
pub async fn async_write_json(p: &'static Path, j: Arc<RwLock<JsonFile>>) -> Result<()> {
    let async_writer = tokio::task::spawn_blocking(move || {
        let guard = j.read(); // .expect("Mutex should be unlocked before trying to lock again");
        write_json(&*p, &*guard)
    });
    async_writer.await?
}

pub fn read_json(path: &Path) -> Result<JsonFile> {
    let rdr = File::options()
        // TODO: Check if write access is required 
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
pub async fn async_read_json(path: &'static Path) -> Result<JsonFile> {
    let async_reader = tokio::task::spawn_blocking(|| read_json(path));
    async_reader.await?
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
        if after == before {
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
            .ok_or(Err::PhonebookEntry("id does not exist".into()))
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
    pub fn get_by_id_sorted(&mut self, id: PersonID) -> Option<Person> {
        self.sort();
        match self.phonebook.binary_search_by_key(&id, |p| p.id).ok() {
            Some(index) => Some(&self.phonebook[index]).cloned(),
            None => None,
        }

        // self.phonebook.iter().find(|p| p.id == id)
    }
    /// get_by_id using binary search but without taking a &mut access to JsonFile
    /// We can perform a binary search because the only way our phonebook
    /// is unsorted is during either initialization or during manually tweaking of the file
    /// after it has been created and populated. The `generate_id` function ensures that
    /// ids are unique and they are created in a linear sequence so as to preserve sort order
    /// Not having a `&mut` reference means that our `RwLock` doesn't require to get a `RwWrtierGuard`
    /// on our `RwLock` which is good for performance.  
    ///
    /// Note: Our JsonFile, in memory is always sorted.
    pub fn get_by_id(&self, id: PersonID) -> Option<Person> {
        if let Some(index) = self.phonebook.binary_search_by_key(&id, |p| p.id).ok() {
            return Some(&self.phonebook[index]).cloned();
        }
        None
    }

    pub fn get_by_name(&self, name: &str) -> Option<Person> {
        // Throwaway the error
        let (p, index) = self.check_if_name_exists(name).ok()?;
        if !p {return None;}
        self.phonebook.get(index.unwrap()).cloned()
    }

    pub fn print_phonebook(&self) {
        let entries = self.phonebook.iter();
        println!("❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯❯");
        for person in entries {
            println!("{person}");
        }
        println!("❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮❮");
    }

    /// Add to a phonebook only if that name is unique
    pub fn add_to_phonebook(&mut self, mut p: Person) -> Result<()> {
        // Handle bad requests such as an `id` not being in their default state 0_u128
        if self.get_by_id(p.id).is_some() {
            log::warn!("Person with id {} already exists in the phonebook", p.id);
            return Err(Err::PhonebookEntry("Person ID already exists".into()))
                .with_context(|| format!("Person with id {} already exists, please do not provide an id", p.id));
        }
        let id = self.generate_id();
        p.id = id;
        if !self.check_if_name_exists(&p.name)?.0 {
            self.phonebook.push(p);
        } else {
            log::warn!("Name {} already exists in the phonebook. Names must be unique", &p.name);
            return Err(Err::PhonebookEntry("Duplicate name".into()))
                .with_context(|| format!("Person with name {} already exists, Names must be unique", p.name));
        }
        Ok(())
    }
    /// Sort the phonebook by id
    pub fn sort(&mut self) {
        // if self.phonebook.iter().is_sorted_by_key(|p| p.id) {
        //     return;
        // }
        self.phonebook.sort_unstable_by_key(|p| p.id);
        log::info!("Phonebook sorted by id");
    }

    // TODO: Currently we do not assign missing ids i.e. ids that were deleted do not 
    // get assinged to newly added entries. Let's fix this
    fn generate_id(&self) -> PersonID {
        let max_phonebook_id = self
            .phonebook
            .iter()
            .max_by_key(|person| (**person).id)
            .and_then(|person| Some(person.id))
            .unwrap_or(<PersonID>::default());
        /* IDs should start with 1 incase this phonebook is empty */
        // Generates a very large id
        let mut candidate = if max_phonebook_id == 0 { 1 } else { max_phonebook_id + 1 };
        while matches!(self.phonebook.iter().next(), Some(person) if person.id == candidate ) {
            // This debug should practically never log
            log::debug!("candidate ID collision found");
            candidate = rand::random::<PersonID>();
        }
        candidate
    }

    fn check_if_name_exists(&self, new_name: &str) -> Result<(bool, Option<usize>)> {
        let new_name = new_name.trim().to_lowercase();
        let mut new_name = new_name.split_whitespace();
        // Important to get length before calling any `next`|`next_back`
        let new_name_len = new_name.size_hint();
        let (new_fname, new_lname) = (
            new_name
                .next()
                // previously ok_or which is a const fn
                .ok_or_else(|| Err::PhonebookEntry("First name missing".into()))
                .with_context(|| {
                    let err = "Phonebook entry should have a first name";
                    log::warn!("{err}");
                    err
                })?,
            new_name.next_back(),
        );
        // We take the size_hint before calling `next` and `next_back` on new_name and name
        // exactly once
        let pos = self
            .phonebook
            .iter()
            .map(|person| person.name.trim().to_lowercase())
            .position(|pname| {
                let mut name = pname.split_whitespace();
                let name_len = name.size_hint();
                // compare upper bounds
                name_len.1 == new_name_len.1
                    && name.next().expect("Safe to unwrap pre-existing fname") == new_fname
                    && matches!(name.next_back(), Some(lname) if matches!(new_lname, Some(n) if n == lname))
            });
        Ok((pos.is_some(), pos))
    }
}

#[tokio::test]
async fn test_methods() -> Result<()> {
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
    write_json(&path, &json_file)?;

    debug_assert_eq!(None, json_file.get_by_id(10));
    Ok(())
}
