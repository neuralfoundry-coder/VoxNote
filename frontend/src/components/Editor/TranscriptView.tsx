import { useEffect, useRef } from "react";
import { useRecordingStore } from "../../stores/recordingStore";
import { useTauriEvent } from "../../hooks/useTauriIPC";
import type { Segment } from "../../lib/types";

const SPEAKER_COLORS = [
  { bg: "rgba(99, 102, 241, 0.08)", border: "rgba(99, 102, 241, 0.2)", text: "#6366f1", dot: "#6366f1" },
  { bg: "rgba(6, 182, 212, 0.08)", border: "rgba(6, 182, 212, 0.2)", text: "#06b6d4", dot: "#06b6d4" },
  { bg: "rgba(245, 158, 11, 0.08)", border: "rgba(245, 158, 11, 0.2)", text: "#f59e0b", dot: "#f59e0b" },
  { bg: "rgba(16, 185, 129, 0.08)", border: "rgba(16, 185, 129, 0.2)", text: "#10b981", dot: "#10b981" },
  { bg: "rgba(244, 63, 94, 0.08)", border: "rgba(244, 63, 94, 0.2)", text: "#f43f5e", dot: "#f43f5e" },
];

interface Props {
  noteId: string;
}

export function TranscriptView({ noteId }: Props) {
  const { segments, addSegment, state: recordingState } = useRecordingStore();
  const bottomRef = useRef<HTMLDivElement>(null);
  const speakerMap = useRef<Map<string, number>>(new Map());

  useTauriEvent<Segment>("stt:segment", (segment) => {
    if (segment.note_id === noteId) {
      addSegment(segment);
    }
  });

  useEffect(() => {
    if (recordingState === "recording") {
      bottomRef.current?.scrollIntoView({ behavior: "smooth" });
    }
  }, [segments.length, recordingState]);

  const getSpeakerColor = (speakerId: string | null) => {
    if (!speakerId) return null;
    if (!speakerMap.current.has(speakerId)) {
      speakerMap.current.set(speakerId, speakerMap.current.size % SPEAKER_COLORS.length);
    }
    return SPEAKER_COLORS[speakerMap.current.get(speakerId)!];
  };

  const filtered = segments.filter((s) => s.note_id === noteId);

  if (filtered.length === 0) {
    return (
      <div className="flex items-center gap-3 py-4">
        {recordingState === "recording" ? (
          <>
            <div className="flex items-end gap-0.5 h-5">
              {[...Array(7)].map((_, i) => (
                <div key={i} className="wave-bar" />
              ))}
            </div>
            <span className="text-sm animate-shimmer" style={{ color: "var(--vn-text-tertiary)" }}>
              Listening...
            </span>
          </>
        ) : (
          <p className="text-sm" style={{ color: "var(--vn-text-tertiary)" }}>
            No transcript yet. Start recording to see live transcription.
          </p>
        )}
      </div>
    );
  }

  // Group consecutive segments by speaker
  let currentSpeaker: string | null | undefined;

  return (
    <div className="space-y-1">
      {filtered.map((segment, idx) => {
        const isNewSpeaker = segment.speaker_id !== currentSpeaker;
        currentSpeaker = segment.speaker_id;
        const color = getSpeakerColor(segment.speaker_id);

        return (
          <div key={segment.id} className="animate-float-in" style={{ animationDelay: `${Math.min(idx * 30, 300)}ms` }}>
            {/* Speaker header (when speaker changes) */}
            {isNewSpeaker && segment.speaker_id && color && (
              <div className="flex items-center gap-2 mt-4 mb-1.5">
                <span className="w-2 h-2 rounded-full" style={{ background: color.dot }} />
                <span className="text-xs font-semibold" style={{ color: color.text }}>
                  {segment.speaker_id}
                </span>
                <span className="text-[10px]" style={{ color: "var(--vn-text-tertiary)" }}>
                  {formatTimestamp(segment.start_ms)}
                </span>
              </div>
            )}

            {/* Segment bubble */}
            <div className="flex gap-3 group">
              {/* Timestamp gutter */}
              {!segment.speaker_id && (
                <span className="text-[10px] font-mono mt-1.5 shrink-0 w-12 text-right opacity-0 group-hover:opacity-100 transition-opacity"
                      style={{ color: "var(--vn-text-tertiary)" }}>
                  {formatTimestamp(segment.start_ms)}
                </span>
              )}

              {/* Text */}
              <div
                className="rounded-xl px-3 py-2 max-w-[85%] transition-glass"
                style={{
                  background: color?.bg || "var(--vn-bg-glass)",
                  borderLeft: color ? `3px solid ${color.border}` : "none",
                }}
              >
                <p className="text-[13px] leading-relaxed" style={{ color: "var(--vn-text-primary)" }}>
                  {segment.text}
                </p>
              </div>
            </div>
          </div>
        );
      })}

      {/* Live indicator */}
      {recordingState === "recording" && (
        <div className="flex items-center gap-2 pt-2">
          <div className="flex items-end gap-0.5 h-4">
            {[...Array(5)].map((_, i) => (
              <div key={i} className="wave-bar" style={{ opacity: 0.5 }} />
            ))}
          </div>
          <span className="text-[11px]" style={{ color: "var(--vn-text-tertiary)" }}>
            Transcribing...
          </span>
        </div>
      )}

      <div ref={bottomRef} />
    </div>
  );
}

function formatTimestamp(ms: number): string {
  const totalSecs = Math.floor(ms / 1000);
  const mins = Math.floor(totalSecs / 60);
  const secs = totalSecs % 60;
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}
