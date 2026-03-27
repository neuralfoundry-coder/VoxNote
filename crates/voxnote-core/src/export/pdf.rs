use super::ExportData;

/// PDF 내보내기 (간단한 텍스트 기반)
///
/// 프로덕션에서는 printpdf 또는 genpdf crate를 사용합니다.
/// 현재는 Markdown을 생성하여 외부 변환기에 위임합니다.
pub fn export_pdf(data: &ExportData) -> Result<Vec<u8>, String> {
    // Markdown을 생성하고 UTF-8 바이트로 반환
    // 실제 PDF 생성은 genpdf crate 연동 시 구현
    let md = super::markdown::export_markdown(data);
    Ok(md.into_bytes())
}
