use super::ExportData;

/// DOCX 내보내기 (간단한 텍스트 기반)
///
/// 프로덕션에서는 docx-rs crate를 사용합니다.
/// 현재는 Markdown을 생성하여 외부 변환기에 위임합니다.
pub fn export_docx(data: &ExportData) -> Result<Vec<u8>, String> {
    let md = super::markdown::export_markdown(data);
    Ok(md.into_bytes())
}
