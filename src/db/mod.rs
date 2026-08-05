pub mod predefined;
pub mod queries;
pub mod schema;

pub use predefined::get_predefined_queries;
pub use queries::Database;

use crate::error::Result;
use rusqlite::Connection;
use std::fs;
use std::path::Path;
use tracing::info;

pub fn open_database<P: AsRef<Path>>(path: P) -> Result<Database> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    info!("Opening SQLite database at {:?}", path.as_ref());
    let conn = Connection::open(path)?;
    schema::init_database(&conn)?;

    Ok(Database::new(conn))
}
