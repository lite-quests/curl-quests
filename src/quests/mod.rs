use rusqlite::Connection;
use rusqlite::types::Value;
use serde::Deserialize;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

pub enum VerifyResult {
    Pass,
    Fail(String),
}

pub struct DbCheck {
    pub sql: String,
    pub expected: String,
    pub error: String,
}

pub struct InputCheck {
    pub contains: Vec<String>,
    pub error: String,
}

pub struct QuestSetup {
    pub seed: Vec<String>,
}

#[allow(dead_code)]
pub struct QuestServer {
    pub port: u16,
}

pub struct QuestVerify {
    pub checks: Vec<DbCheck>,
    pub input: Option<InputCheck>,
}

pub struct Quest {
    pub id: usize,
    pub title: String,
    pub instructions: String,
    pub solutions: Vec<String>,
    pub submit_prompt: Option<String>,
    pub setup: Option<QuestSetup>,
    #[allow(dead_code)]
    pub server: Option<QuestServer>,
    pub verify: QuestVerify,
    pub folder_path: PathBuf,
}

impl Quest {
    /// Verify quest DB state by running check queries against the quest database.
    pub fn verify_db(&self, quest_db_path: &Path) -> VerifyResult {
        if self.verify.checks.is_empty() {
            return VerifyResult::Pass;
        }
        let conn = match Connection::open(quest_db_path) {
            Ok(c) => c,
            Err(e) => return VerifyResult::Fail(format!("Could not open quest DB: {e}")),
        };
        for check in &self.verify.checks {
            let result: Result<String, _> = conn.query_row(&check.sql, [], |row| {
                let val: Value = row.get(0)?;
                Ok(match val {
                    Value::Integer(i) => i.to_string(),
                    Value::Real(f) => f.to_string(),
                    Value::Text(s) => s,
                    Value::Blob(b) => String::from_utf8_lossy(&b).to_string(),
                    Value::Null => "NULL".to_string(),
                })
            });
            match result {
                Ok(val) if val == check.expected => continue,
                Ok(_) | Err(_) => return VerifyResult::Fail(check.error.clone()),
            }
        }
        VerifyResult::Pass
    }

    /// Verify user's submitted answer text.
    pub fn verify_input(&self, input: &str) -> VerifyResult {
        if let Some(ref check) = self.verify.input {
            let lower = input.to_lowercase();
            for pattern in &check.contains {
                if !lower.contains(&pattern.to_lowercase()) {
                    return VerifyResult::Fail(check.error.clone());
                }
            }
        }
        VerifyResult::Pass
    }
}

// ---------------------------------------------------------------------------
// TOML schema (private only used during loading)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct QuestToml {
    id: usize,
    title: String,
    instructions: String,
    solutions: Vec<String>,
    submit_prompt: Option<String>,
    setup: Option<SetupToml>,
    server: Option<ServerToml>,
    verify: VerifyToml,
}

#[derive(Deserialize)]
struct SetupToml {
    seed: Vec<String>,
}

#[derive(Deserialize)]
struct ServerToml {
    port: u16,
}

#[derive(Deserialize)]
struct VerifyToml {
    checks: Option<Vec<DbCheckToml>>,
    input: Option<InputCheckToml>,
}

#[derive(Deserialize)]
struct DbCheckToml {
    sql: String,
    expected: String,
    error: String,
}

#[derive(Deserialize)]
struct InputCheckToml {
    contains: Vec<String>,
    error: String,
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

/// Load all quests from `quests_dir`. Each sub-folder must contain a
/// `quest.toml`. Folders that cannot be parsed are silently skipped.
/// The returned vec is sorted by quest id.
pub fn load_quests(quests_dir: &Path) -> Vec<Quest> {
    let mut quests = Vec::new();

    let entries = match std::fs::read_dir(quests_dir) {
        Ok(e) => e,
        Err(_) => return quests,
    };

    let mut dirs: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    dirs.sort_by_key(|e| e.file_name());

    for entry in dirs {
        let folder = entry.path();
        let content = match std::fs::read_to_string(folder.join("quest.toml")) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let parsed: QuestToml = match toml::from_str(&content) {
            Ok(p) => p,
            Err(e) => {
                let _ = std::fs::write(folder.join("error.log"), format!("TOML Parse Error: {}", e));
                continue;
            }
        };

        let setup = parsed.setup.map(|s| QuestSetup { seed: s.seed });
        let server = parsed.server.map(|s| QuestServer { port: s.port });
        let verify = QuestVerify {
            checks: parsed
                .verify
                .checks
                .unwrap_or_default()
                .into_iter()
                .map(|c| DbCheck {
                    sql: c.sql,
                    expected: c.expected,
                    error: c.error,
                })
                .collect(),
            input: parsed.verify.input.map(|i| InputCheck {
                contains: i.contains,
                error: i.error,
            }),
        };

        quests.push(Quest {
            id: parsed.id,
            title: parsed.title,
            instructions: parsed.instructions,
            solutions: parsed.solutions,
            submit_prompt: parsed.submit_prompt,
            setup,
            server,
            verify,
            folder_path: folder,
        });
    }

    quests.sort_by_key(|q| q.id);
    quests
}
