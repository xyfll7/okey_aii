use crate::ai::config::Provider;
use crate::ai::state::HistoryItem;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use rig::message::Message;

pub type Db = Arc<Mutex<Connection>>;

#[derive(Clone, serde::Serialize)]
pub struct SessionMeta {
    pub session_id: String,
    pub provider: Provider,
    pub model: String,
    pub preset_id: String,
    #[serde(serialize_with = "crate::ai::state::serialize_systemtime_millis")]
    pub created_at: SystemTime,
    #[serde(serialize_with = "crate::ai::state::serialize_systemtime_millis")]
    pub update_at: SystemTime,
    pub title: String,
}

fn to_millis(t: SystemTime) -> u64 {
    t.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn from_millis(ms: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_millis(ms)
}

fn provider_to_str(p: Provider) -> &'static str {
    match p {
        Provider::OpenAI => "OpenAI",
        Provider::Anthropic => "Anthropic",
        Provider::DeepSeek => "DeepSeek",
    }
}

fn provider_from_str(s: &str) -> Result<Provider, String> {
    match s {
        "OpenAI" => Ok(Provider::OpenAI),
        "Anthropic" => Ok(Provider::Anthropic),
        "DeepSeek" => Ok(Provider::DeepSeek),
        _ => Err(format!("unknown provider in db: {s}")),
    }
}

pub fn open(path: PathBuf) -> Result<Db, String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let conn = Connection::open(&path).map_err(|e| e.to_string())?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sessions (
            id         TEXT PRIMARY KEY,
            provider   TEXT NOT NULL,
            model      TEXT NOT NULL,
            preset_id  TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            update_at  INTEGER NOT NULL DEFAULT 0,
            title      TEXT NOT NULL,
            status     TEXT NOT NULL DEFAULT 'active'
        );
        CREATE TABLE IF NOT EXISTS messages (
            id         TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            message    TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id, created_at);",
    )
    .map_err(|e| e.to_string())?;
    Ok(Arc::new(Mutex::new(conn)))
}

pub fn insert_session(
    db: &Db,
    session_id: &str,
    provider: Provider,
    model: &str,
    preset_id: &str,
    created_at: SystemTime,
    update_at: SystemTime,
    title: &str,
) -> Result<(), String> {
    let conn = db.lock().unwrap();
    conn.execute(
        "INSERT OR REPLACE INTO sessions (id, provider, model, preset_id, created_at, update_at, title)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            session_id,
            provider_to_str(provider),
            model,
            preset_id,
            to_millis(created_at) as i64,
            to_millis(update_at) as i64,
            title
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_session(
    db: &Db,
    session_id: &str,
    provider: Provider,
    model: &str,
    update_at: SystemTime,
) -> Result<(), String> {
    let conn = db.lock().unwrap();
    conn.execute(
        "UPDATE sessions SET provider = ?1, model = ?2, update_at = ?3 WHERE id = ?4",
        params![
            provider_to_str(provider),
            model,
            to_millis(update_at) as i64,
            session_id
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn set_session_archived(db: &Db, session_id: &str) -> Result<(), String> {
    let conn = db.lock().unwrap();
    conn.execute(
        "UPDATE sessions SET status = 'archived' WHERE id = ?1",
        params![session_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_session_meta(db: &Db, session_id: &str) -> Result<Option<SessionMeta>, String> {
    let conn = db.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, provider, model, preset_id, created_at, update_at, title FROM sessions WHERE id = ?1")
        .map_err(|e| e.to_string())?;
    let mut rows = stmt
        .query_map(params![session_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    match rows.next().transpose().map_err(|e| e.to_string())? {
        Some((id, provider, model, preset_id, created_at, update_at, title)) => Ok(Some(SessionMeta {
            session_id: id,
            provider: provider_from_str(&provider)?,
            model,
            preset_id,
            created_at: from_millis(created_at as u64),
            update_at: from_millis(update_at as u64),
            title,
        })),
        None => Ok(None),
    }
}

pub fn list_session_meta(db: &Db) -> Result<Vec<SessionMeta>, String> {
    let conn = db.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, provider, model, preset_id, created_at, update_at, title FROM sessions ORDER BY update_at")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        let (id, provider, model, preset_id, created_at, update_at, title) =
            row.map_err(|e| e.to_string())?;
        out.push(SessionMeta {
            session_id: id,
            provider: provider_from_str(&provider)?,
            model,
            preset_id,
            created_at: from_millis(created_at as u64),
            update_at: from_millis(update_at as u64),
            title,
        });
    }
    Ok(out)
}

pub fn insert_message(db: &Db, session_id: &str, item: &HistoryItem) -> Result<(), String> {
    let conn = db.lock().unwrap();
    let message_json = serde_json::to_string(&item.message).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO messages (id, session_id, created_at, message) VALUES (?1, ?2, ?3, ?4)",
        params![item.id, session_id, to_millis(item.created_at) as i64, message_json],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_history(db: &Db, session_id: &str) -> Result<Vec<HistoryItem>, String> {
    let conn = db.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, created_at, message FROM messages WHERE session_id = ?1 ORDER BY created_at, rowid")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![session_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        let (id, created_at, message_json) = row.map_err(|e| e.to_string())?;
        let message: Message = serde_json::from_str(&message_json).map_err(|e| e.to_string())?;
        out.push(HistoryItem {
            id,
            created_at: from_millis(created_at as u64),
            message,
        });
    }
    Ok(out)
}

pub fn clear_history(db: &Db, session_id: &str) -> Result<(), String> {
    let conn = db.lock().unwrap();
    conn.execute("DELETE FROM messages WHERE session_id = ?1", params![session_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}


