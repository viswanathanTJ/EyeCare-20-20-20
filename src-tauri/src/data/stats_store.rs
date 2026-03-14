use rusqlite::{Connection, params};
use std::path::PathBuf;

fn db_path() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")));
    path.push("eye2020");
    std::fs::create_dir_all(&path).ok();
    path.push("eye2020.db");
    path
}

pub struct StatsStore {
    conn: Connection,
}

impl StatsStore {
    pub fn open() -> Result<Self, String> {
        let conn = Connection::open(db_path()).map_err(|e| e.to_string())?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS stats (
                date            TEXT PRIMARY KEY,
                breaks_completed INTEGER NOT NULL DEFAULT 0,
                breaks_skipped   INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS daily_streak (
                id              INTEGER PRIMARY KEY CHECK (id = 1),
                current_streak  INTEGER NOT NULL DEFAULT 0,
                longest_streak  INTEGER NOT NULL DEFAULT 0,
                last_break_date TEXT
            );
            INSERT OR IGNORE INTO daily_streak (id, current_streak, longest_streak) VALUES (1, 0, 0);",
        )
        .map_err(|e| e.to_string())?;
        Ok(Self { conn })
    }

    // --- Settings persistence ---

    pub fn save_setting(&self, key: &str, value: &str) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
                params![key, value],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_setting(&self, key: &str) -> Option<String> {
        self.conn
            .query_row("SELECT value FROM settings WHERE key = ?1", params![key], |row| {
                row.get(0)
            })
            .ok()
    }

    // --- Statistics ---

    fn today() -> String {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    }

    pub fn record_break_completed(&self) -> Result<(), String> {
        let today = Self::today();
        self.conn
            .execute(
                "INSERT INTO stats (date, breaks_completed, breaks_skipped) VALUES (?1, 1, 0)
                 ON CONFLICT(date) DO UPDATE SET breaks_completed = breaks_completed + 1",
                params![today],
            )
            .map_err(|e| e.to_string())?;
        self.update_streak(&today)?;
        Ok(())
    }

    pub fn record_break_skipped(&self) -> Result<(), String> {
        let today = Self::today();
        self.conn
            .execute(
                "INSERT INTO stats (date, breaks_skipped, breaks_completed) VALUES (?1, 1, 0)
                 ON CONFLICT(date) DO UPDATE SET breaks_skipped = breaks_skipped + 1",
                params![today],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn update_streak(&self, today: &str) -> Result<(), String> {
        let (current, longest, last_date): (i64, i64, Option<String>) = self
            .conn
            .query_row(
                "SELECT current_streak, longest_streak, last_break_date FROM daily_streak WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| e.to_string())?;

        let yesterday = (chrono::Local::now() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();

        let new_streak = if last_date.as_deref() == Some(today) {
            current // already counted today
        } else if last_date.as_deref() == Some(&yesterday) {
            current + 1
        } else {
            1 // streak reset
        };

        let new_longest = std::cmp::max(longest, new_streak);

        self.conn
            .execute(
                "UPDATE daily_streak SET current_streak = ?1, longest_streak = ?2, last_break_date = ?3 WHERE id = 1",
                params![new_streak, new_longest, today],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_today_stats(&self) -> (i64, i64) {
        let today = Self::today();
        self.conn
            .query_row(
                "SELECT breaks_completed, breaks_skipped FROM stats WHERE date = ?1",
                params![today],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap_or((0, 0))
    }

    pub fn get_streak(&self) -> (i64, i64) {
        self.conn
            .query_row(
                "SELECT current_streak, longest_streak FROM daily_streak WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap_or((0, 0))
    }

    pub fn get_total_breaks(&self) -> i64 {
        self.conn
            .query_row("SELECT COALESCE(SUM(breaks_completed), 0) FROM stats", [], |row| {
                row.get(0)
            })
            .unwrap_or(0)
    }
}
