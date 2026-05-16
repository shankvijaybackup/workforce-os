use rusqlite::{params, Connection, Result};
use std::path::Path;
use std::fs;

pub struct TelemetryCache {
    conn: Connection,
}

impl TelemetryCache {
    pub fn initialize() -> Result<Self> {
        // Enforce strict local pathing. 
        // NT AUTHORITY\LocalService must have read/write access to this directory.
        let storage_dir = Path::new("C:\\ProgramData\\WorkforceOS");
        if !storage_dir.exists() {
            let _ = fs::create_dir_all(storage_dir); // Mute panic for local mac cross-compilation tests
        }

        let db_path = storage_dir.join("telemetry_cache.db");
        // Fallback for local Darwin test compilation
        let conn = if db_path.parent().map(|p| p.exists()).unwrap_or(false) {
            Connection::open(db_path)?
        } else {
            Connection::open("telemetry_cache.db")?
        };
        
        // Optimize for concurrent background reads while the hooks write payload data
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS queued_payloads (
                internal_id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id TEXT UNIQUE NOT NULL,
                timestamp TEXT NOT NULL,
                user_id TEXT NOT NULL,
                ciphertext TEXT NOT NULL,
                iv TEXT NOT NULL,
                auth_tag TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn spool_payload(&self, event_id: &str, ts: &str, user_id: &str, cipher: &str, iv: &str, tag: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO queued_payloads (event_id, timestamp, user_id, ciphertext, iv, auth_tag) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![event_id, ts, user_id, cipher, iv, tag],
        )?;
        Ok(())
    }

    pub fn extract_batch(&self, batch_size: u32) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT event_id FROM queued_payloads ORDER BY internal_id ASC LIMIT ?1"
        )?;
        
        let batch = stmt.query_map(params![batch_size], |row| {
            row.get(0)
        })?.filter_map(Result::ok).collect();

        Ok(batch)
    }

    pub fn purge_acknowledged(&self, event_id: &str) -> Result<()> {
        // Executed STRICTLY after receiving HTTP 200 OK from AWS API Gateway
        self.conn.execute(
            "DELETE FROM queued_payloads WHERE event_id = ?1",
            params![event_id],
        )?;
        Ok(())
    }
}
