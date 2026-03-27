use crate::diarize::SpeakerSegment;
use crate::models::Segment;

/// 화자 분리 결과를 전사 세그먼트에 태깅
pub fn tag_speakers(segments: &mut [Segment], speaker_segments: &[SpeakerSegment]) {
    for segment in segments.iter_mut() {
        // 시간 구간이 가장 많이 겹치는 화자를 할당
        let mut best_overlap = 0i64;
        let mut best_speaker = None;

        for ss in speaker_segments {
            let overlap_start = segment.start_ms.max(ss.start_ms);
            let overlap_end = segment.end_ms.min(ss.end_ms);
            let overlap = (overlap_end - overlap_start).max(0);

            if overlap > best_overlap {
                best_overlap = overlap;
                best_speaker = Some(ss.speaker_id.clone());
            }
        }

        if let Some(speaker) = best_speaker {
            segment.speaker_id = Some(speaker);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speaker_tagging() {
        let mut segments = vec![
            Segment::new("n1", "Hello", 0, 2000),
            Segment::new("n1", "World", 2000, 4000),
        ];

        let speaker_segments = vec![
            SpeakerSegment { speaker_id: "A".into(), start_ms: 0, end_ms: 2500, confidence: 0.9 },
            SpeakerSegment { speaker_id: "B".into(), start_ms: 2500, end_ms: 5000, confidence: 0.8 },
        ];

        tag_speakers(&mut segments, &speaker_segments);
        assert_eq!(segments[0].speaker_id, Some("A".to_string()));
        assert_eq!(segments[1].speaker_id, Some("B".to_string()));
    }
}
