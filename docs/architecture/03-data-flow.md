# VoxNote 데이터 흐름 파이프라인

> 최종 갱신: 2026-03-27 | 버전: 0.1.0-draft

---

## 1. 실시간 전사 파이프라인

마이크 또는 시스템 사운드에서 캡처된 오디오가 텍스트로 변환되어 Frontend에 표시되기까지의 전체 흐름이다.

```mermaid
flowchart TD
    MIC["마이크 / 시스템 사운드"]
    CPAL["cpal 캡처<br/>(48kHz f32 stereo)"]
    RB["RingBuffer<br/>(lock-free crossbeam)"]
    RS["Resampler<br/>(rubato: 16kHz mono)"]
    VAD["VAD<br/>(Silero / webrtc-vad)"]
    ACC["Accumulator<br/>(2-3초 슬라이딩 윈도우<br/>+ 0.5초 오버랩)"]
    WHISPER["whisper.cpp<br/>(+ initial_prompt<br/>+ 커스텀 단어장)"]
    PP["PostProcessor"]
    LLM_PASS["LLM 경량 패스<br/>(오탈자 교정)"]
    AHO["Aho-Corasick<br/>(고유명사 매칭)"]
    DIAR["SpeakerDiarizer<br/>(화자 태깅)"]
    TS["TimestampAlign<br/>(타임스탬프 정렬)"]
    EMIT["Tauri Event Emit<br/>(stt:segment)"]
    FE["Frontend UI 업데이트"]
    DB["SQLite 저장"]
    SUMQ["요약 큐<br/>(1-2분 축적 후 전달)"]

    MIC --> CPAL
    CPAL --> RB
    RB --> RS
    RS --> VAD
    VAD -->|"음성 구간"| ACC
    VAD -->|"무음"| DISCARD["폐기"]
    ACC --> WHISPER
    WHISPER --> PP

    PP --> LLM_PASS
    PP --> AHO
    PP --> DIAR
    PP --> TS

    LLM_PASS --> MERGE["병합"]
    AHO --> MERGE
    DIAR --> MERGE
    TS --> MERGE

    MERGE --> EMIT
    EMIT --> FE
    MERGE --> DB
    MERGE --> SUMQ

    style MIC fill:#78909c,stroke:#37474f,color:#fff
    style CPAL fill:#ff7043,stroke:#d84315,color:#fff
    style RB fill:#ffa726,stroke:#e65100,color:#000
    style RS fill:#ffca28,stroke:#f57f17,color:#000
    style VAD fill:#66bb6a,stroke:#2e7d32,color:#fff
    style WHISPER fill:#ab47bc,stroke:#6a1b9a,color:#fff
    style PP fill:#ffa726,stroke:#e65100,color:#000
    style LLM_PASS fill:#5c6bc0,stroke:#283593,color:#fff
    style DIAR fill:#ef5350,stroke:#b71c1c,color:#fff
    style EMIT fill:#61dafb,stroke:#21a1c4,color:#000
    style FE fill:#61dafb,stroke:#21a1c4,color:#000
    style DB fill:#8d6e63,stroke:#4e342e,color:#fff
    style SUMQ fill:#26a69a,stroke:#00695c,color:#fff
```

### 파이프라인 성능 지표

| 구간 | 목표 지연 | 비고 |
|------|----------|------|
| cpal 캡처 → RingBuffer | < 5ms | lock-free, 실시간 스레드 |
| RingBuffer → 리샘플링 | < 10ms | rubato 비동기 처리 |
| VAD 판정 | < 2ms | 30ms 프레임 단위 |
| Whisper 추론 (3초 청크) | < 500ms | large-v3-turbo 기준, Metal 가속 |
| PostProcessor 전체 | < 100ms | 병렬 처리 |
| **End-to-End 지연** | **< 1초** | 발화 종료 → 텍스트 표시 |

---

## 2. 요약/문서 생성 파이프라인

전사된 텍스트를 축적하여 주기적으로 요약을 생성하는 파이프라인이다.

```mermaid
flowchart TD
    SUMQ["요약 큐<br/>(1-2분 분량 축적)"]
    PB["PromptBuilder"]
    SYS["시스템 프롬프트<br/>(역할 + 출력 형식 지시)"]
    PREV["이전 요약<br/>(컨텍스트 연속성)"]
    CURR["현재 전사 텍스트<br/>(새로운 내용)"]
    TPL["템플릿<br/>(회의록/브레인스토밍/강의 등)"]
    LLAMA["llama.cpp<br/>(GGUF Q4_K_M)"]
    STREAM["StreamingToken"]
    EMIT["Tauri Event Emit<br/>(llm:token)"]
    FE["Frontend 실시간 렌더링"]
    SPP["SummaryPostProcessor<br/>(포맷 정리 + 검증)"]
    DB["SQLite 저장"]

    SUMQ --> PB
    SYS --> PB
    PREV --> PB
    CURR --> PB
    TPL --> PB

    PB -->|"조합된 프롬프트"| LLAMA
    LLAMA --> STREAM
    STREAM --> EMIT
    EMIT --> FE
    STREAM --> SPP
    SPP -->|"최종 요약"| DB
    SPP -->|"다음 사이클 이전요약"| PREV

    style SUMQ fill:#26a69a,stroke:#00695c,color:#fff
    style PB fill:#ffa726,stroke:#e65100,color:#000
    style SYS fill:#bdbdbd,stroke:#616161,color:#000
    style PREV fill:#90a4ae,stroke:#455a64,color:#000
    style CURR fill:#a5d6a7,stroke:#388e3c,color:#000
    style TPL fill:#ce93d8,stroke:#7b1fa2,color:#000
    style LLAMA fill:#5c6bc0,stroke:#283593,color:#fff
    style STREAM fill:#7986cb,stroke:#303f9f,color:#fff
    style EMIT fill:#61dafb,stroke:#21a1c4,color:#000
    style FE fill:#61dafb,stroke:#21a1c4,color:#000
    style SPP fill:#ffb74d,stroke:#e65100,color:#000
    style DB fill:#8d6e63,stroke:#4e342e,color:#fff
```

### PromptBuilder 구성

```
┌──────────────────────────────────────────────┐
│ [시스템 프롬프트]                               │
│ 당신은 회의록 요약 전문 AI입니다.                  │
│ 한국어로 작성하고 Markdown 형식을 따르세요.         │
├──────────────────────────────────────────────┤
│ [이전 요약] (있는 경우)                          │
│ ## 이전 회의 요약                                │
│ - 핵심 안건 1 ...                               │
│ - 결정 사항 ...                                  │
├──────────────────────────────────────────────┤
│ [현재 전사 텍스트]                               │
│ [00:15:30] 화자A: 다음 분기 계획에 대해...         │
│ [00:15:45] 화자B: 예산은 어떻게...                │
├──────────────────────────────────────────────┤
│ [템플릿 지시]                                    │
│ 다음 항목으로 정리: 핵심 안건, 결정 사항,            │
│ 액션 아이템, 미해결 이슈                           │
└──────────────────────────────────────────────┘
```

### 요약 생성 파라미터

| 파라미터 | 값 | 설명 |
|---------|-----|------|
| 모델 | Q4_K_M 양자화 | 속도/품질 균형 |
| 컨텍스트 윈도우 | 8192 토큰 | 충분한 전사 텍스트 수용 |
| Temperature | 0.3 | 낮은 창의성, 높은 정확성 |
| Top-P | 0.9 | 누적 확률 기반 샘플링 |
| 반복 페널티 | 1.1 | 반복 표현 억제 |
| GBNF Grammar | Markdown 구조 | 출력 형식 강제 |

---

## 3. CRDT 동기화 시퀀스

여러 디바이스 간 노트를 E2EE 상태로 동기화하는 흐름이다. 서버는 암호화된 데이터를 중계할 뿐, 내용을 열람할 수 없다.

```mermaid
sequenceDiagram
    participant A as Device A
    participant S as Sync Relay Server
    participant B as Device B

    Note over A: 사용자가 노트 편집

    A->>A: y-crdt 델타 생성
    A->>A: age(X25519) 암호화

    A->>S: 암호화된 델타 전송 (WebSocket)

    Note over S: 복호화 불가<br/>암호화된 상태 그대로 보관

    alt Device B 온라인
        S->>B: 암호화된 델타 중계
        B->>B: age 복호화 (개인키)
        B->>B: y-crdt 머지 (자동 충돌 해소)
        B->>B: UI 업데이트
    else Device B 오프라인
        Note over S: 서버 30일 버퍼링<br/>(암호화 상태 유지)
        B-->>S: 재접속
        S->>B: 누적 암호화 델타 전송
        B->>B: age 복호화
        B->>B: y-crdt 순차 머지
        B->>B: UI 업데이트
    end

    Note over A,B: 양방향 동기화 — B의 변경도 동일 경로로 A에 전달
```

### 동기화 프로토콜 상세

```mermaid
flowchart LR
    subgraph "발신 디바이스"
        EDIT["노트 편집<br/>(텍스트/메타데이터)"]
        YDOC["y-crdt Doc"]
        DELTA["델타 추출<br/>(encode_state_as_update)"]
        ENC["age 암호화<br/>(X25519 + ChaCha20)"]
        WS_SEND["WebSocket 전송"]
    end

    subgraph "Sync Relay"
        RECV["수신"]
        STORE["암호화 버퍼<br/>(30일 TTL)"]
        FWD["대상 디바이스에 전달"]
    end

    subgraph "수신 디바이스"
        WS_RECV["WebSocket 수신"]
        DEC["age 복호화"]
        MERGE["y-crdt 머지<br/>(apply_update)"]
        UI["UI 반영"]
    end

    EDIT --> YDOC --> DELTA --> ENC --> WS_SEND
    WS_SEND --> RECV --> STORE --> FWD
    FWD --> WS_RECV --> DEC --> MERGE --> UI

    style ENC fill:#ef5350,stroke:#b71c1c,color:#fff
    style DEC fill:#66bb6a,stroke:#2e7d32,color:#fff
    style STORE fill:#78909c,stroke:#37474f,color:#fff
```

### 충돌 해소 전략

| 시나리오 | CRDT 해소 방식 |
|---------|---------------|
| 같은 위치 동시 삽입 | 삽입 순서를 클라이언트 ID 기준으로 결정 |
| 같은 텍스트 동시 삭제 | 멱등 삭제 — 이미 삭제된 항목 무시 |
| 메타데이터 동시 수정 | Last-Writer-Wins Register (LWW) |
| 오프라인 장기 편집 후 머지 | 모든 연산이 교환 법칙 충족, 순서 무관 머지 |

---

## 4. Ask VoxNote (RAG) 파이프라인

저장된 회의 전사 텍스트를 기반으로 사용자의 자연어 질문에 답변하는 RAG(Retrieval-Augmented Generation) 파이프라인이다.

### 4.1 인덱싱 파이프라인 (오프라인)

새로운 전사/요약이 저장될 때 비동기적으로 임베딩을 생성하여 벡터 검색을 준비한다.

```mermaid
flowchart TD
    SRC["전사 텍스트 / 요약"]
    CHUNK["청크 분할<br/>(512 토큰, 25% 오버랩)"]
    META["메타데이터 첨부<br/>(회의 ID, 화자, 시간)"]
    EMB["임베딩 생성<br/>(로컬: all-MiniLM-L6-v2<br/>또는 클라우드: text-embedding-3-small)"]
    STORE["SQLite BLOB 저장<br/>(chunk_text + embedding + metadata)"]

    SRC --> CHUNK
    CHUNK --> META
    META --> EMB
    EMB --> STORE

    style SRC fill:#a5d6a7,stroke:#388e3c,color:#000
    style CHUNK fill:#ffca28,stroke:#f57f17,color:#000
    style META fill:#90a4ae,stroke:#455a64,color:#000
    style EMB fill:#ab47bc,stroke:#6a1b9a,color:#fff
    style STORE fill:#8d6e63,stroke:#4e342e,color:#fff
```

### 4.2 질의 파이프라인 (온라인)

사용자가 질문을 입력하면 관련 청크를 검색하고, LLM에 주입하여 답변을 생성한다.

```mermaid
flowchart TD
    Q["사용자 질문<br/>예: 지난주 회의에서 예산 관련 결정은?"]
    Q_EMB["질문 임베딩 생성"]
    SEARCH["코사인 유사도 검색<br/>(SQLite 전체 스캔 또는 HNSW)"]
    TOPK["Top-K 청크 선택<br/>(K=5, 유사도 임계값 0.7)"]
    FTS["FTS5 키워드 보강 검색<br/>(하이브리드: 벡터 + 키워드)"]
    RERANK["Re-Rank<br/>(교차 점수 기반 재정렬)"]
    CTX["LLM 컨텍스트 조립"]
    SYS_P["시스템 프롬프트:<br/>제공된 회의 내용을 기반으로<br/>정확하게 답변하세요"]
    CHUNKS["검색된 청크들<br/>(출처: 회의명, 날짜, 화자)"]
    USER_Q["사용자 질문"]
    LLM["LLM 추론<br/>(llama.cpp / Cloud API)"]
    ANS["답변 생성<br/>(+ 출처 인용 포함)"]
    FE["Frontend 표시<br/>(답변 + 출처 링크)"]

    Q --> Q_EMB
    Q_EMB --> SEARCH
    SEARCH --> TOPK
    Q --> FTS
    FTS --> TOPK
    TOPK --> RERANK
    RERANK --> CTX

    SYS_P --> CTX
    CHUNKS --> CTX
    USER_Q --> CTX

    CTX --> LLM
    LLM --> ANS
    ANS --> FE

    style Q fill:#42a5f5,stroke:#1565c0,color:#fff
    style Q_EMB fill:#ab47bc,stroke:#6a1b9a,color:#fff
    style SEARCH fill:#ffca28,stroke:#f57f17,color:#000
    style TOPK fill:#ffa726,stroke:#e65100,color:#000
    style FTS fill:#66bb6a,stroke:#2e7d32,color:#fff
    style RERANK fill:#ec407a,stroke:#ad1457,color:#fff
    style CTX fill:#bdbdbd,stroke:#616161,color:#000
    style LLM fill:#5c6bc0,stroke:#283593,color:#fff
    style ANS fill:#26a69a,stroke:#00695c,color:#fff
    style FE fill:#61dafb,stroke:#21a1c4,color:#000
```

### RAG 파라미터

| 파라미터 | 값 | 설명 |
|---------|-----|------|
| 청크 크기 | 512 토큰 | 의미 단위 보존과 검색 정밀도 균형 |
| 청크 오버랩 | 25% (128 토큰) | 청크 경계에서의 문맥 손실 방지 |
| 임베딩 모델 (로컬) | all-MiniLM-L6-v2 | 384차원, ONNX Runtime |
| 임베딩 모델 (클라우드) | text-embedding-3-small | 1536차원, OpenAI API |
| Top-K | 5 | 최대 5개 관련 청크 선택 |
| 유사도 임계값 | 0.7 | 이 이상만 컨텍스트에 포함 |
| 하이브리드 가중치 | 벡터 0.7 + 키워드 0.3 | 의미 검색과 키워드 검색 병합 |
| LLM Temperature | 0.2 | 사실 기반 답변을 위해 낮게 설정 |
| 최대 컨텍스트 | 4096 토큰 | 시스템 프롬프트 + 청크 + 질문 |

---

## 부록: 전체 데이터 흐름 요약

```mermaid
flowchart LR
    subgraph "입력"
        A1["마이크"]
        A2["시스템 사운드"]
        A3["파일 임포트"]
    end

    subgraph "처리"
        B1["Audio Pipeline"]
        B2["STT Engine"]
        B3["PostProcessor"]
        B4["LLM Engine"]
        B5["RAG Pipeline"]
    end

    subgraph "저장"
        C1["SQLite<br/>(전사, 요약, 임베딩)"]
        C2["redb<br/>(세션 상태, 캐시)"]
        C3["파일시스템<br/>(모델, 오디오)"]
    end

    subgraph "출력"
        D1["실시간 전사 UI"]
        D2["요약 문서"]
        D3["RAG 답변"]
        D4["TTS 음성"]
        D5["E2EE 동기화"]
    end

    A1 --> B1
    A2 --> B1
    A3 --> B2
    B1 --> B2
    B2 --> B3
    B3 --> B4
    B3 --> C1
    B4 --> C1
    C1 --> B5

    B3 --> D1
    B4 --> D2
    B5 --> D3
    B4 --> D4
    C1 --> D5

    style B1 fill:#ff7043,stroke:#d84315,color:#fff
    style B2 fill:#ab47bc,stroke:#6a1b9a,color:#fff
    style B3 fill:#ffa726,stroke:#e65100,color:#000
    style B4 fill:#5c6bc0,stroke:#283593,color:#fff
    style B5 fill:#26a69a,stroke:#00695c,color:#fff
    style C1 fill:#8d6e63,stroke:#4e342e,color:#fff
```
