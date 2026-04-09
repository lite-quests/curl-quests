use rusqlite::{params, Connection, Result};
use std::collections::HashSet;
use std::path::PathBuf;

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open() -> Result<Self> {
        std::fs::create_dir_all("data").ok();
        let conn = Connection::open("data/app.db")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS completions (
                quest_id INTEGER PRIMARY KEY,
                completed_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )?;
        Ok(Self { conn })
    }

    pub fn completed_quests(&self) -> HashSet<usize> {
        let mut stmt = self.conn.prepare("SELECT quest_id FROM completions").unwrap();
        stmt.query_map([], |row| row.get::<_, i64>(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .map(|id| id as usize)
            .collect()
    }

    pub fn mark_completed(&self, quest_id: usize) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO completions (quest_id) VALUES (?1)",
            params![quest_id as i64],
        )?;
        Ok(())
    }

    pub fn reset_all(&self) -> Result<()> {
        self.conn.execute("DELETE FROM completions", [])?;
        Ok(())
    }
}

/// Returns the path to the quest-specific database.
pub fn quest_db_path() -> PathBuf {
    PathBuf::from("data/quest.db")
}

/// Wipe and re-seed the quest database with the given SQL statements.
pub fn setup_quest_db(seed_sql: &[String]) -> std::result::Result<(), String> {
    let path = quest_db_path();
    let _ = std::fs::remove_file(&path);
    std::fs::create_dir_all("data").ok();
    let conn = Connection::open(&path).map_err(|e| format!("Failed to create quest DB: {e}"))?;
    // Ensure writes are fully flushed to disk before the connection closes.
    conn.execute_batch("PRAGMA synchronous=FULL;").ok();
    // Wrap all seed statements in one transaction so they commit atomically.
    conn.execute_batch("BEGIN;")
        .map_err(|e| format!("Failed to begin transaction: {e}"))?;
    for sql in seed_sql {
        conn.execute_batch(sql)
            .map_err(|e| format!("Seed SQL failed: {e}"))?;
    }
    conn.execute_batch("COMMIT;")
        .map_err(|e| format!("Failed to commit seed: {e}"))?;
    Ok(())
}
