// Modified by ACRYL, 2026
//! Migration v3 → v4: Add image_index table.

use rusqlite::Connection;

use acrylmicrovm_shared::errors::{AcrylMicrovmError, AcrylMicrovmResult};

use super::{Migration, db_err};
use crate::db::schema;

pub(crate) struct AddImageIndex;

impl Migration for AddImageIndex {
    fn source_version(&self) -> i32 {
        3
    }
    fn target_version(&self) -> i32 {
        4
    }
    fn description(&self) -> &str {
        "Add image_index table"
    }

    fn run(&self, conn: &Connection, _home_dir: Option<&std::path::Path>) -> AcrylMicrovmResult<()> {
        db_err!(conn.execute_batch(schema::IMAGE_INDEX_TABLE))?;
        Ok(())
    }
}
