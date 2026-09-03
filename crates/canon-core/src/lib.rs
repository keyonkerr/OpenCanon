//! Domain rules and pure computation. No filesystem, clock, or environment access.

pub mod compute;
pub mod error;
pub mod lifecycle;
pub mod model;
pub mod ops;

pub use error::{Error, SlugConflict};
pub use model::{Atom, ComposedDoc, Freshness, Score, Status, Timestamp};
