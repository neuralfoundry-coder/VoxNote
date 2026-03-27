use std::collections::HashMap;

/// Aho-Corasick 기반 고유명사 매칭
///
/// vocabulary 테이블의 단어 목록으로 전사 텍스트의 오탈자를 보정합니다.
pub struct ProperNounMatcher {
    /// original → replacement 매핑
    replacements: HashMap<String, String>,
}

impl ProperNounMatcher {
    pub fn new() -> Self {
        Self {
            replacements: HashMap::new(),
        }
    }

    pub fn add_term(&mut self, original: &str, replacement: &str) {
        self.replacements
            .insert(original.to_lowercase(), replacement.to_string());
    }

    pub fn load_vocabulary(&mut self, terms: Vec<(String, String)>) {
        for (original, replacement) in terms {
            self.add_term(&original, &replacement);
        }
    }

    /// 텍스트에서 고유명사를 찾아 치환
    pub fn process(&self, text: &str) -> String {
        let mut result = text.to_string();
        // 길이 역순으로 정렬하여 긴 매칭 우선
        let mut sorted: Vec<_> = self.replacements.iter().collect();
        sorted.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        for (original, replacement) in sorted {
            // 대소문자 무시 치환
            let lower = result.to_lowercase();
            if let Some(pos) = lower.find(original.as_str()) {
                let end = pos + original.len();
                result = format!("{}{}{}", &result[..pos], replacement, &result[end..]);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proper_noun_replacement() {
        let mut matcher = ProperNounMatcher::new();
        matcher.add_term("복스노트", "VoxNote");
        matcher.add_term("클로드", "Claude");

        let result = matcher.process("복스노트 앱에서 클로드를 사용합니다");
        assert!(result.contains("VoxNote"));
        assert!(result.contains("Claude"));
    }
}
