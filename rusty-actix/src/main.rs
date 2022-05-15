//! A rusty-server (a rust equivalent for json server), like the json-server has two parts
//! A web server that exposes RESTful endpoints
//! And a file reader writer that can read and manipulate a json file

// use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    #[error("Json error")]
    Json(#[from] serde_json::error::Error),
}

// TODO : Bring io and serde_json errors under a single error type
fn main() -> Result<(), Err> {
    // let path = Path::new("../phonebook.json");
    let path = Path::new("mock.json");
    let rdr = File::options()
        .write(true)
        .read(true)
        .open(path)
        .map_err(|err| Err::Io(err))?;
    // The content of the IO stream is deserialized directly from the stream without being buffered in memory by serde_json.
    // let phonebook = serde_json::from_reader::<File, Value>(rdr)?;
    // https://github.com/serde-rs/json/issues/160
    // https://github.com/paritytech/substrate/pull/10137
    // let buf_rdr = BufReader::new(rdr);
    // let phonebook = serde_json::from_reader::<BufReader<File>, Value>(buf_rdr)?;
    // Apparently reading the entire file into memory is the fastest way to deserialize i.e. `from_slice` and `from_str` methods
    // are faster than the `from_reader` method
    let bytes = unsafe { memmap2::Mmap::map(&rdr).map_err(|err| Err::Io(err))? };

    let json_file =
        serde_json::from_slice::<Value>(&bytes).map_err(|err| Err::Json(err))?;
    #[allow(unused_mut)]
    let mut phonebook = &json_file["phonebook"];

    println!("{:?}", phonebook[1]);

    Ok(())
}
