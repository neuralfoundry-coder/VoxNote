use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Mutex;
use tracing::{debug, info};

use super::migration;
use crate::error::StorageError;
use crate::models::{Folder, Note, NoteStatus, Segment};

/// 전문검색 결과
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    pub segment_id: String,
    pub note_id: String,
    pub text: String,
    pub highlight: String,
    pub rank: f64,
}

/// SQLite 저장소
pub struct SqliteStore {
    conn: Mutex<Connection>,
}

impl SqliteStore {
    /// 파일 기반 DB 열기 (없으면 생성)
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                StorageError::Migration(format!("Failed to create data dir: {}", e))
            })?;
        }

        let conn = Connection::open(path)?;
        Self::init(conn)
    }

    /// 인메모리 DB (테스트용)
    pub fn open_in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        Self::init(conn)
    }

    fn init(conn: Connection) -> Result<Self, StorageError> {
        // WAL 모드 활성화
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        // 마이그레이션 실행
        migration::run_migrations(&conn)?;
        info!("SQLite store initialized");

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    // ── Notes CRUD ─────────────────────────────────────────────────

    pub fn insert_note(&self, note: &Note) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO notes (id, title, status, folder_id, duration_ms, language, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                note.id,
                note.title,
                status_to_str(&note.status),
                note.folder_id,
                note.duration_ms,
                note.language,
                note.created_at.to_rfc3339(),
                note.updated_at.to_rfc3339(),
            ],
        )?;
        debug!("Inserted note: {}", note.id);
        Ok(())
    }

    pub fn get_note(&self, id: &str) -> Result<Option<Note>, StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, title, status, folder_id, duration_ms, language, created_at, updated_at
             FROM notes WHERE id = ?1",
            params![id],
            |row| {
                Ok(Note {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    status: str_to_status(&row.get::<_, String>(2)?),
                    folder_id: row.get(3)?,
                    duration_ms: row.get(4)?,
                    language: row.get(5)?,
                    created_at: parse_datetime(&row.get::<_, String>(6)?),
                    updated_at: parse_datetime(&row.get::<_, String>(7)?),
                })
            },
        )
        .optional()
        .map_err(StorageError::from)
    }

    pub fn list_notes(&self, folder_id: Option<&str>) -> Result<Vec<Note>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = if folder_id.is_some() {
            conn.prepare(
                "SELECT id, title, status, folder_id, duration_ms, language, created_at, updated_at
                 FROM notes WHERE folder_id = ?1 ORDER BY created_at DESC",
            )?
        } else {
            conn.prepare(
                "SELECT id, title, status, folder_id, duration_ms, language, created_at, updated_at
                 FROM notes ORDER BY created_at DESC",
            )?
        };

        let params_slice: Vec<Box<dyn rusqlite::types::ToSql>> = if let Some(fid) = folder_id {
            vec![Box::new(fid.to_string())]
        } else {
            vec![]
        };

        let rows = stmt.query_map(rusqlite::params_from_iter(params_slice.iter()), |row| {
            Ok(Note {
                id: row.get(0)?,
                title: row.get(1)?,
                status: str_to_status(&row.get::<_, String>(2)?),
                folder_id: row.get(3)?,
                duration_ms: row.get(4)?,
                language: row.get(5)?,
                created_at: parse_datetime(&row.get::<_, String>(6)?),
                updated_at: parse_datetime(&row.get::<_, String>(7)?),
            })
        })?;

        let mut notes = Vec::new();
        for row in rows {
            notes.push(row?);
        }
        Ok(notes)
    }

    pub fn update_note(&self, note: &Note) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute(
            "UPDATE notes SET title=?2, status=?3, folder_id=?4, duration_ms=?5, language=?6, updated_at=?7
             WHERE id=?1",
            params![
                note.id,
                note.title,
                status_to_str(&note.status),
                note.folder_id,
                note.duration_ms,
                note.language,
                note.updated_at.to_rfc3339(),
            ],
        )?;
        if affected == 0 {
            return Err(StorageError::NotFound(format!("Note {}", note.id)));
        }
        Ok(())
    }

    pub fn delete_note(&self, id: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
        if affected == 0 {
            return Err(StorageError::NotFound(format!("Note {}", id)));
        }
        Ok(())
    }

    // ── Transcripts ────────────────────────────────────────────────

    pub fn insert_segment(&self, segment: &Segment) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO transcripts (id, note_id, timestamp_ms, end_ms, text, speaker_id, confidence)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                segment.id,
                segment.note_id,
                segment.start_ms,
                segment.end_ms,
                segment.text,
                segment.speaker_id,
                segment.confidence,
            ],
        )?;
        Ok(())
    }

    pub fn get_segments(&self, note_id: &str) -> Result<Vec<Segment>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, note_id, text, timestamp_ms, end_ms, speaker_id, confidence
             FROM transcripts WHERE note_id = ?1 ORDER BY timestamp_ms ASC",
        )?;
        let rows = stmt.query_map(params![note_id], |row| {
            Ok(Segment {
                id: row.get(0)?,
                note_id: row.get(1)?,
                text: row.get(2)?,
                start_ms: row.get(3)?,
                end_ms: row.get(4)?,
                speaker_id: row.get(5)?,
                confidence: row.get(6)?,
            })
        })?;
        let mut segments = Vec::new();
        for row in rows {
            segments.push(row?);
        }
        Ok(segments)
    }

    // ── Folders ────────────────────────────────────────────────────

    pub fn insert_folder(&self, folder: &Folder) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO folders (id, name, parent_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                folder.id,
                folder.name,
                folder.parent_id,
                folder.created_at.to_rfc3339(),
                folder.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn list_folders(&self, parent_id: Option<&str>) -> Result<Vec<Folder>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = if parent_id.is_some() {
            conn.prepare(
                "SELECT id, name, parent_id, created_at, updated_at
                 FROM folders WHERE parent_id = ?1 ORDER BY name",
            )?
        } else {
            conn.prepare(
                "SELECT id, name, parent_id, created_at, updated_at
                 FROM folders WHERE parent_id IS NULL ORDER BY name",
            )?
        };

        let params_slice: Vec<Box<dyn rusqlite::types::ToSql>> = if let Some(pid) = parent_id {
            vec![Box::new(pid.to_string())]
        } else {
            vec![]
        };

        let rows = stmt.query_map(rusqlite::params_from_iter(params_slice.iter()), |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                created_at: parse_datetime(&row.get::<_, String>(3)?),
                updated_at: parse_datetime(&row.get::<_, String>(4)?),
            })
        })?;
        let mut folders = Vec::new();
        for row in rows {
            folders.push(row?);
        }
        Ok(folders)
    }

    pub fn delete_folder(&self, id: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM folders WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ── Full-Text Search (FTS5) ────────────────────────────────────

    pub fn search_transcripts(&self, query: &str) -> Result<Vec<SearchResult>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT t.id, t.note_id, t.text,
                    highlight(transcript_fts, 0, '<mark>', '</mark>'),
                    rank
             FROM transcript_fts
             JOIN transcripts t ON t.rowid = transcript_fts.rowid
             WHERE transcript_fts MATCH ?1
             ORDER BY rank
             LIMIT 50",
        )?;

        let rows = stmt.query_map(params![query], |row| {
            Ok(SearchResult {
                segment_id: row.get(0)?,
                note_id: row.get(1)?,
                text: row.get(2)?,
                highlight: row.get(3)?,
                rank: row.get(4)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    // ── Provider Config ────────────────────────────────────────────

    pub fn insert_provider_config(&self, config: &ProviderConfigRow) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO provider_config (id, engine_type, provider, model_id, endpoint, is_active, config_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                config.id,
                config.engine_type,
                config.provider,
                config.model_id,
                config.endpoint,
                config.is_active as i32,
                config.config_json,
                config.created_at.to_rfc3339(),
                config.updated_at.to_rfc3339(),
            ],
        )?;
        debug!("Inserted provider_config: {} ({})", config.provider, config.engine_type);
        Ok(())
    }

    pub fn get_provider_configs(&self) -> Result<Vec<ProviderConfigRow>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, engine_type, provider, model_id, endpoint, is_active, config_json, created_at, updated_at
             FROM provider_config ORDER BY engine_type, provider",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ProviderConfigRow {
                id: row.get(0)?,
                engine_type: row.get(1)?,
                provider: row.get(2)?,
                model_id: row.get(3)?,
                endpoint: row.get(4)?,
                is_active: row.get::<_, i32>(5)? != 0,
                config_json: row.get(6)?,
                created_at: parse_datetime(&row.get::<_, String>(7)?),
                updated_at: parse_datetime(&row.get::<_, String>(8)?),
            })
        })?;
        let mut configs = Vec::new();
        for row in rows {
            configs.push(row?);
        }
        Ok(configs)
    }

    pub fn get_active_provider(&self, engine_type: &str) -> Result<Option<ProviderConfigRow>, StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, engine_type, provider, model_id, endpoint, is_active, config_json, created_at, updated_at
             FROM provider_config WHERE engine_type = ?1 AND is_active = 1 LIMIT 1",
            params![engine_type],
            |row| {
                Ok(ProviderConfigRow {
                    id: row.get(0)?,
                    engine_type: row.get(1)?,
                    provider: row.get(2)?,
                    model_id: row.get(3)?,
                    endpoint: row.get(4)?,
                    is_active: true,
                    config_json: row.get(6)?,
                    created_at: parse_datetime(&row.get::<_, String>(7)?),
                    updated_at: parse_datetime(&row.get::<_, String>(8)?),
                })
            },
        )
        .optional()
        .map_err(StorageError::from)
    }

    pub fn update_provider_config(&self, config: &ProviderConfigRow) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute(
            "UPDATE provider_config SET engine_type=?2, provider=?3, model_id=?4, endpoint=?5,
             is_active=?6, config_json=?7, updated_at=?8 WHERE id=?1",
            params![
                config.id,
                config.engine_type,
                config.provider,
                config.model_id,
                config.endpoint,
                config.is_active as i32,
                config.config_json,
                config.updated_at.to_rfc3339(),
            ],
        )?;
        if affected == 0 {
            return Err(StorageError::NotFound(format!("ProviderConfig {}", config.id)));
        }
        Ok(())
    }

    pub fn upsert_provider_config(&self, config: &ProviderConfigRow) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO provider_config (id, engine_type, provider, model_id, endpoint, is_active, config_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                engine_type=excluded.engine_type, provider=excluded.provider,
                model_id=excluded.model_id, endpoint=excluded.endpoint,
                is_active=excluded.is_active, config_json=excluded.config_json,
                updated_at=excluded.updated_at",
            params![
                config.id,
                config.engine_type,
                config.provider,
                config.model_id,
                config.endpoint,
                config.is_active as i32,
                config.config_json,
                config.created_at.to_rfc3339(),
                config.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn delete_provider_config(&self, id: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM provider_config WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// 특정 engine_type의 모든 provider를 비활성화
    pub fn deactivate_providers(&self, engine_type: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE provider_config SET is_active = 0, updated_at = datetime('now') WHERE engine_type = ?1",
            params![engine_type],
        )?;
        Ok(())
    }

    // ── Summaries ──────────────────────────────────────────────────

    pub fn insert_summary(&self, summary: &SummaryRow) -> Result<(), StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO summaries (id, note_id, template_id, content, model_used, provider, version, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                summary.id,
                summary.note_id,
                summary.template_id,
                summary.content,
                summary.model_used,
                summary.provider,
                summary.version,
                summary.created_at.to_rfc3339(),
            ],
        )?;
        debug!("Inserted summary for note: {}", summary.note_id);
        Ok(())
    }

    pub fn get_summaries(&self, note_id: &str) -> Result<Vec<SummaryRow>, StorageError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, note_id, template_id, content, model_used, provider, version, created_at
             FROM summaries WHERE note_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![note_id], |row| {
            Ok(SummaryRow {
                id: row.get(0)?,
                note_id: row.get(1)?,
                template_id: row.get(2)?,
                content: row.get(3)?,
                model_used: row.get(4)?,
                provider: row.get(5)?,
                version: row.get(6)?,
                created_at: parse_datetime(&row.get::<_, String>(7)?),
            })
        })?;
        let mut summaries = Vec::new();
        for row in rows {
            summaries.push(row?);
        }
        Ok(summaries)
    }

    pub fn get_latest_summary(&self, note_id: &str) -> Result<Option<SummaryRow>, StorageError> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, note_id, template_id, content, model_used, provider, version, created_at
             FROM summaries WHERE note_id = ?1 ORDER BY created_at DESC LIMIT 1",
            params![note_id],
            |row| {
                Ok(SummaryRow {
                    id: row.get(0)?,
                    note_id: row.get(1)?,
                    template_id: row.get(2)?,
                    content: row.get(3)?,
                    model_used: row.get(4)?,
                    provider: row.get(5)?,
                    version: row.get(6)?,
                    created_at: parse_datetime(&row.get::<_, String>(7)?),
                })
            },
        )
        .optional()
        .map_err(StorageError::from)
    }
}

// ── Row Types ─────────────────────────────────────────────────────

/// Provider 설정 행
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderConfigRow {
    pub id: String,
    pub engine_type: String,
    pub provider: String,
    pub model_id: Option<String>,
    pub endpoint: Option<String>,
    pub is_active: bool,
    pub config_json: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl ProviderConfigRow {
    pub fn new(engine_type: &str, provider: &str) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            engine_type: engine_type.to_string(),
            provider: provider.to_string(),
            model_id: None,
            endpoint: None,
            is_active: false,
            config_json: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// 요약 행
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SummaryRow {
    pub id: String,
    pub note_id: String,
    pub template_id: Option<String>,
    pub content: String,
    pub model_used: Option<String>,
    pub provider: Option<String>,
    pub version: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl SummaryRow {
    pub fn new(note_id: &str, content: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            note_id: note_id.to_string(),
            template_id: None,
            content: content.to_string(),
            model_used: None,
            provider: None,
            version: 1,
            created_at: chrono::Utc::now(),
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────

fn status_to_str(status: &NoteStatus) -> &str {
    match status {
        NoteStatus::Recording => "recording",
        NoteStatus::Transcribing => "transcribing",
        NoteStatus::Summarizing => "summarizing",
        NoteStatus::Done => "done",
        NoteStatus::Error => "error",
    }
}

fn str_to_status(s: &str) -> NoteStatus {
    match s {
        "recording" => NoteStatus::Recording,
        "transcribing" => NoteStatus::Transcribing,
        "summarizing" => NoteStatus::Summarizing,
        "done" => NoteStatus::Done,
        _ => NoteStatus::Error,
    }
}

fn parse_datetime(s: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_crud() {
        let store = SqliteStore::open_in_memory().unwrap();

        let note = Note::new("Test Meeting");
        store.insert_note(&note).unwrap();

        let fetched = store.get_note(&note.id).unwrap().unwrap();
        assert_eq!(fetched.title, "Test Meeting");
        assert_eq!(fetched.status, NoteStatus::Recording);

        let notes = store.list_notes(None).unwrap();
        assert_eq!(notes.len(), 1);

        store.delete_note(&note.id).unwrap();
        assert!(store.get_note(&note.id).unwrap().is_none());
    }

    #[test]
    fn test_segment_insert_and_search() {
        let store = SqliteStore::open_in_memory().unwrap();

        let note = Note::new("Search Test");
        store.insert_note(&note).unwrap();

        let seg = Segment::new(&note.id, "VoxNote 회의록 자동 요약 테스트", 0, 3000);
        store.insert_segment(&seg).unwrap();

        let segments = store.get_segments(&note.id).unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "VoxNote 회의록 자동 요약 테스트");

        // FTS5 검색
        let results = store.search_transcripts("회의록").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].highlight.contains("<mark>"));
    }

    #[test]
    fn test_folder_crud() {
        let store = SqliteStore::open_in_memory().unwrap();

        let folder = Folder::new("Work", None);
        store.insert_folder(&folder).unwrap();

        let folders = store.list_folders(None).unwrap();
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0].name, "Work");

        store.delete_folder(&folder.id).unwrap();
        assert!(store.list_folders(None).unwrap().is_empty());
    }
}
