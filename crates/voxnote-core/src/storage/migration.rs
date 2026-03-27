use rusqlite::Connection;

use crate::error::StorageError;

/// DB 스키마 마이그레이션 실행
pub fn run_migrations(conn: &Connection) -> Result<(), StorageError> {
    // 버전 추적 테이블
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );"
    )?;

    let current = get_version(conn)?;

    if current < 1 {
        migrate_v1(conn)?;
    }

    Ok(())
}

fn get_version(conn: &Connection) -> Result<i32, StorageError> {
    let version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    Ok(version)
}

/// Migration v1: 전체 9 테이블 + FTS5
///
/// Phase 2에서 활용할 테이블(summaries, embeddings 등)도
/// 이 시점에서 미리 생성하여 후속 마이그레이션 없이 데이터를 적재합니다.
fn migrate_v1(conn: &Connection) -> Result<(), StorageError> {
    conn.execute_batch(
        "
        -- 폴더 (계층 구조)
        CREATE TABLE IF NOT EXISTS folders (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            parent_id   TEXT REFERENCES folders(id) ON DELETE SET NULL,
            created_at  TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- 노트
        CREATE TABLE IF NOT EXISTS notes (
            id          TEXT PRIMARY KEY,
            title       TEXT NOT NULL,
            status      TEXT NOT NULL DEFAULT 'recording',
            folder_id   TEXT REFERENCES folders(id) ON DELETE SET NULL,
            duration_ms INTEGER,
            language    TEXT,
            created_at  TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- 전사 텍스트
        CREATE TABLE IF NOT EXISTS transcripts (
            id            TEXT PRIMARY KEY,
            note_id       TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
            timestamp_ms  INTEGER NOT NULL,
            end_ms        INTEGER NOT NULL DEFAULT 0,
            text          TEXT NOT NULL,
            speaker_id    TEXT,
            confidence    REAL
        );

        -- FTS5 전문검색 가상 테이블
        CREATE VIRTUAL TABLE IF NOT EXISTS transcript_fts USING fts5(
            text,
            content='transcripts',
            content_rowid='rowid',
            tokenize='unicode61 remove_diacritics 2'
        );

        -- FTS5 자동 동기화 트리거
        CREATE TRIGGER IF NOT EXISTS transcript_fts_insert AFTER INSERT ON transcripts BEGIN
            INSERT INTO transcript_fts(rowid, text) VALUES (new.rowid, new.text);
        END;

        CREATE TRIGGER IF NOT EXISTS transcript_fts_delete AFTER DELETE ON transcripts BEGIN
            INSERT INTO transcript_fts(transcript_fts, rowid, text) VALUES('delete', old.rowid, old.text);
        END;

        CREATE TRIGGER IF NOT EXISTS transcript_fts_update AFTER UPDATE ON transcripts BEGIN
            INSERT INTO transcript_fts(transcript_fts, rowid, text) VALUES('delete', old.rowid, old.text);
            INSERT INTO transcript_fts(rowid, text) VALUES (new.rowid, new.text);
        END;

        -- 요약 (Phase 2 활용)
        CREATE TABLE IF NOT EXISTS summaries (
            id          TEXT PRIMARY KEY,
            note_id     TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
            template_id TEXT,
            content     TEXT NOT NULL,
            model_used  TEXT,
            provider    TEXT,
            version     INTEGER NOT NULL DEFAULT 1,
            created_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- 임베딩 (Phase 2 RAG 활용)
        CREATE TABLE IF NOT EXISTS embeddings (
            id          TEXT PRIMARY KEY,
            note_id     TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
            chunk_index INTEGER NOT NULL,
            chunk_text  TEXT NOT NULL,
            vector      BLOB NOT NULL,
            model_id    TEXT,
            created_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- 사용자 단어장 (Phase 2 활용)
        CREATE TABLE IF NOT EXISTS vocabulary (
            id          TEXT PRIMARY KEY,
            original    TEXT NOT NULL,
            replacement TEXT NOT NULL,
            domain      TEXT,
            created_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- AI Provider 설정 (Phase 2 활용)
        CREATE TABLE IF NOT EXISTS provider_config (
            id          TEXT PRIMARY KEY,
            engine_type TEXT NOT NULL,
            provider    TEXT NOT NULL,
            model_id    TEXT,
            endpoint    TEXT,
            is_active   INTEGER NOT NULL DEFAULT 0,
            config_json TEXT,
            created_at  TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- 요약 템플릿 (Phase 2 활용)
        CREATE TABLE IF NOT EXISTS templates (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            description TEXT,
            prompt      TEXT NOT NULL,
            is_builtin  INTEGER NOT NULL DEFAULT 0,
            created_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- 화자 프로필 (Phase 2 활용)
        CREATE TABLE IF NOT EXISTS speaker_profiles (
            id          TEXT PRIMARY KEY,
            name        TEXT,
            embedding   BLOB NOT NULL,
            sample_count INTEGER NOT NULL DEFAULT 1,
            created_at  TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- 인덱스
        CREATE INDEX IF NOT EXISTS idx_notes_folder ON notes(folder_id);
        CREATE INDEX IF NOT EXISTS idx_notes_created ON notes(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_transcripts_note ON transcripts(note_id);
        CREATE INDEX IF NOT EXISTS idx_transcripts_time ON transcripts(note_id, timestamp_ms);
        CREATE INDEX IF NOT EXISTS idx_summaries_note ON summaries(note_id);
        CREATE INDEX IF NOT EXISTS idx_embeddings_note ON embeddings(note_id);

        -- 버전 기록
        INSERT INTO schema_version (version) VALUES (1);
        "
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migration_v1() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // 테이블 존재 확인
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"notes".to_string()));
        assert!(tables.contains(&"transcripts".to_string()));
        assert!(tables.contains(&"folders".to_string()));
        assert!(tables.contains(&"summaries".to_string()));
        assert!(tables.contains(&"embeddings".to_string()));
        assert!(tables.contains(&"vocabulary".to_string()));
        assert!(tables.contains(&"provider_config".to_string()));
        assert!(tables.contains(&"templates".to_string()));
        assert!(tables.contains(&"speaker_profiles".to_string()));
        assert!(tables.contains(&"schema_version".to_string()));
    }

    #[test]
    fn test_migration_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap(); // 재실행 시 에러 없어야 함
    }
}
