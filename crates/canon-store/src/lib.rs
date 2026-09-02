//! Atom ↔ `opencanon/atoms/<id>.md`. The only crate that touches the disk.

mod error;
mod io;
mod layout;
mod serialize;

pub use error::Error;
pub use io::Store;
