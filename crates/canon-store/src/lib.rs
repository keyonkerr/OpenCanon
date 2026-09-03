//! Atom ↔ `opencanon/atoms/<id>.md`. The only crate that touches the disk.

mod error;
mod io;
mod layout;
mod serialize;
mod serialize_doc;

pub use error::Error;
pub use io::Store;
