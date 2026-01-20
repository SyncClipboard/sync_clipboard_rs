use rusqlite::{params, Connection, Result};
use clipboard_core::clipboard::ClipboardData;
use std::sync::{Arc, Mutex};

pub struct Database {
    conn: Arc<Mutex<Connection>>,
    max_count: u32,
}

impl Database {
    pub fn new(path: &str, max_count: u32) -> Result<Self> {
        let conn = Connection::open(path)?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY,
                type TEXT NOT NULL,
                content TEXT,
                html TEXT,
                file TEXT,
                hash TEXT,
                device TEXT,
                pinned BOOLEAN DEFAULT 0,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        
        // Attempt migration for existing DBs
        let _ = conn.execute("ALTER TABLE history ADD COLUMN html TEXT", []);
        let _ = conn.execute("ALTER TABLE history ADD COLUMN device TEXT", []);
        let _ = conn.execute("ALTER TABLE history ADD COLUMN pinned BOOLEAN DEFAULT 0", []);

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            max_count,
        })
    }

    pub fn save(&self, data: &ClipboardData) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        match data {
            ClipboardData::Text { content, file, html, device } => {
                conn.execute(
                    "INSERT INTO history (type, content, file, html, device) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params!["Text", content, file, html, device],
                )?;
            }
            ClipboardData::Image { hash, filename, device } => {
                conn.execute(
                    "INSERT INTO history (type, hash, file, device) VALUES (?1, ?2, ?3, ?4)",
                    params!["Image", hash, filename, device],
                )?;
            }
            ClipboardData::File { hash, filename, device } => {
                conn.execute(
                    "INSERT INTO history (type, hash, file, device) VALUES (?1, ?2, ?3, ?4)",
                    params!["File", hash, filename, device],
                )?;
            }
        }

        // Cleanup old history (preserve pinned items)
        if self.max_count > 0 {
            conn.execute(
                "DELETE FROM history WHERE pinned = 0 AND id NOT IN (
                    SELECT id FROM history ORDER BY id DESC LIMIT ?1
                )",
                params![self.max_count],
            )?;
        }

        Ok(())
    }

    pub fn get_latest(&self) -> Result<Option<ClipboardData>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT type, content, file, hash, html, device FROM history ORDER BY id DESC LIMIT 1")?;
        
        let mut rows = stmt.query([])?;
        
        if let Some(row) = rows.next()? {
            let type_: String = row.get(0)?;
            let content: Option<String> = row.get(1)?;
            let file: Option<String> = row.get(2)?;
            let hash: Option<String> = row.get(3)?;
            let html: Option<String> = row.get(4)?;
            let device: Option<String> = row.get(5)?;

            let data = match type_.as_str() {
                "Text" => ClipboardData::Text {
                    content: content.unwrap_or_default(),
                    file,
                    html,
                    device,
                },
                "Image" => ClipboardData::Image {
                    hash,
                    filename: file.unwrap_or_default(),
                    device,
                },
                "File" => ClipboardData::File {
                    hash,
                    filename: file.unwrap_or_default(),
                    device,
                },
                _ => return Ok(None),
            };
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }
    pub fn get_latest_id(&self) -> Result<Option<i64>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id FROM history ORDER BY id DESC LIMIT 1")?;
        let mut rows = stmt.query([])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_history(&self, limit: u32, offset: u32) -> Result<Vec<(i64, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, bool, String)>> {
        let conn = self.conn.lock().unwrap();
        // Return tuple: (id, type, content, file, hash, html, device, pinned, timestamp)
        let mut stmt = conn.prepare(
            "SELECT id, type, content, file, hash, html, device, pinned, timestamp 
             FROM history 
             ORDER BY pinned DESC, id DESC 
             LIMIT ?1 OFFSET ?2"
        )?;
        
        let rows = stmt.query_map(params![limit, offset], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get::<_, Option<bool>>(7)?.unwrap_or(false),
                row.get::<_, String>(8)?,
            ))
        })?;

        let mut history = Vec::new();
        for row in rows {
            history.push(row?);
        }
        Ok(history)
    }

    pub fn delete_history(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM history WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn set_pinned(&self, id: i64, pinned: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE history SET pinned = ?1 WHERE id = ?2", params![pinned, id])?;
        Ok(())
    }
}
