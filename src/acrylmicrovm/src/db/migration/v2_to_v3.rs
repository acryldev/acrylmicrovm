// Modified by ACRYL, 2026
//! Migration v2 → v3: Add name column to box_config.

use rusqlite::Connection;

use acrylmicrovm_shared::errors::{AcrylMicrovmError, AcrylMicrovmResult};

use super::{Migration, db_err};

pub(crate) struct AddNameColumn;

impl Migration for AddNameColumn {
    fn source_version(&self) -> i32 {
        2
    }
    fn target_version(&self) -> i32 {
        3
    }
    fn description(&self) -> &str {
        "Add name column to box_config"
    }

    fn run(&self, conn: &Connection, _home_dir: Option<&std::path::Path>) -> AcrylMicrovmResult<()> {
        db_err!(conn.execute_batch("ALTER TABLE box_config ADD COLUMN name TEXT;"))?;
        db_err!(conn.execute_batch(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_box_config_name_unique ON box_config(name);"
        ))?;
        db_err!(conn.execute_batch(
            "UPDATE box_config SET name = json_extract(json, '$.name') WHERE name IS NULL;"
        ))?;
        Ok(())
    }
}
