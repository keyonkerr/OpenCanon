mod activate;
mod add;
mod compose;
mod delete;
mod edit;
mod get;
mod list;
mod query;

use canon_core::ops::ListFilter;
use canon_core::Timestamp;
use canon_store::Store;
use serde_json::Value;

use crate::error::CliError;

pub fn add(store: &Store) -> Result<Value, CliError> {
    add::run(store)
}

pub fn get(store: &Store, id: &str) -> Result<Value, CliError> {
    get::run(store, id)
}

pub fn list(store: &Store, filter: ListFilter) -> Result<Value, CliError> {
    list::run(store, filter)
}

pub fn query(store: &Store, keywords: &[String], filter: ListFilter) -> Result<Value, CliError> {
    query::run(store, keywords, filter)
}

pub fn compose(store: &Store) -> Result<Value, CliError> {
    compose::run(store)
}

pub fn edit(store: &Store) -> Result<Value, CliError> {
    edit::run(store)
}

pub fn delete(store: &Store, id: &str) -> Result<Value, CliError> {
    delete::run(store, id)
}

pub fn active(store: &Store, now: Timestamp, id: &str) -> Result<Value, CliError> {
    activate::run(store, now, id)
}
