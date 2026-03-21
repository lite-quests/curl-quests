use rusqlite::{params, Connection, Result};
use std::collections::HashSet;

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
