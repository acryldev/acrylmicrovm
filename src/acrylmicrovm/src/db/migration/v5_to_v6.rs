// Modified by ACRYL, 2026
//! Migration v5 → v6: Replace snapshots with box_snapshot table.

use rusqlite::Connection;

use acrylmicrovm_shared::errors::{AcrylMicrovmError, AcrylMicrovmResult};

use super::{Migration, db_err};
use crate::db::schema;

pub(crate) struct ReplaceSnapshots;

impl Migration for ReplaceSnapshots {
    fn source_version(&self) -> i32 {
        5
    }
    fn target_version(&self) -> i32 {
        6
    }
    fn description(&self) -> &str {
        "Replace snapshots with box_snapshot table"
    }

    fn run(
        &self,
        conn: &Connection,
        _home_dir: Option<&std::path::Path>,
    ) -> AcrylMicrovmResult<()> {
        db_err!(conn.execute_batch("DROP TABLE IF EXISTS snapshots;"))?;
        db_err!(conn.execute_batch(schema::BOX_SNAPSHOT_TABLE))?;
        Ok(())
    }
}
