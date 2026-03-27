use yrs::{Doc, ReadTxn, Text, Transact, Update};
use tracing::debug;

/// y-crdt 기반 문서 동기화 관리
pub struct CrdtDocument {
    doc: Doc,
}

impl CrdtDocument {
    pub fn new() -> Self {
        Self { doc: Doc::new() }
    }

    /// 텍스트 내용 설정
    pub fn set_text(&self, key: &str, content: &str) {
        let text = self.doc.get_or_insert_text(key);
        let mut txn = self.doc.transact_mut();
        let len = text.get_string(&txn).len() as u32;
        if len > 0 {
            text.remove_range(&mut txn, 0, len);
        }
        text.insert(&mut txn, 0, content);
    }

    /// 텍스트 내용 가져오기
    pub fn get_text(&self, key: &str) -> String {
        let text = self.doc.get_or_insert_text(key);
        let txn = self.doc.transact();
        text.get_string(&txn)
    }

    /// 상태 벡터 추출 (동기화 기준점)
    pub fn state_vector(&self) -> Vec<u8> {
        let txn = self.doc.transact();
        txn.state_vector().encode_v1()
    }

    /// 델타 추출 (state vector 이후 변경분)
    pub fn encode_diff(&self, remote_sv: &[u8]) -> Result<Vec<u8>, String> {
        let sv = yrs::StateVector::decode_v1(remote_sv)
            .map_err(|e| format!("Invalid state vector: {}", e))?;
        let txn = self.doc.transact();
        Ok(txn.encode_diff_v1(&sv))
    }

    /// 원격 델타 적용
    pub fn apply_update(&self, update: &[u8]) -> Result<(), String> {
        let update = Update::decode_v1(update)
            .map_err(|e| format!("Invalid update: {}", e))?;
        let mut txn = self.doc.transact_mut();
        txn.apply_update(update)
            .map_err(|e| format!("Failed to apply update: {}", e))
    }

    /// 전체 상태 스냅샷
    pub fn encode_state(&self) -> Vec<u8> {
        let txn = self.doc.transact();
        txn.encode_state_as_update_v1(&yrs::StateVector::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crdt_sync() {
        let doc_a = CrdtDocument::new();
        let doc_b = CrdtDocument::new();

        doc_a.set_text("notes", "Hello from A");
        let sv_b = doc_b.state_vector();
        let diff = doc_a.encode_diff(&sv_b).unwrap();
        doc_b.apply_update(&diff).unwrap();

        assert_eq!(doc_b.get_text("notes"), "Hello from A");
    }

    #[test]
    fn test_concurrent_edits() {
        let doc_a = CrdtDocument::new();
        let doc_b = CrdtDocument::new();

        // 초기 동기화
        let state = doc_a.encode_state();
        doc_b.apply_update(&state).unwrap();

        // 동시 편집
        doc_a.set_text("notes", "Edit by A");
        doc_b.set_text("notes", "Edit by B");

        // A → B 동기화
        let sv_b = doc_b.state_vector();
        let diff_a = doc_a.encode_diff(&sv_b).unwrap();
        doc_b.apply_update(&diff_a).unwrap();

        // B → A 동기화
        let sv_a = doc_a.state_vector();
        let diff_b = doc_b.encode_diff(&sv_a).unwrap();
        doc_a.apply_update(&diff_b).unwrap();

        // CRDT는 동일한 최종 상태를 보장
        assert_eq!(doc_a.get_text("notes"), doc_b.get_text("notes"));
    }
}
