use crossbeam::queue::ArrayQueue;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::warn;

/// Lock-free 링버퍼 — 캡처 스레드와 처리 스레드 간 데이터 전달
///
/// 오버플로우 시 오래된 데이터를 드롭하고 경고 로그를 남깁니다.
pub struct AudioRingBuffer {
    queue: Arc<ArrayQueue<Vec<f32>>>,
    dropped_count: Arc<AtomicU64>,
    capacity: usize,
}

impl AudioRingBuffer {
    /// 새 링버퍼 생성
    ///
    /// `capacity`: 최대 보관 가능한 청크 수
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: Arc::new(ArrayQueue::new(capacity)),
            dropped_count: Arc::new(AtomicU64::new(0)),
            capacity,
        }
    }

    /// 프로듀서 핸들 생성 (캡처 스레드용)
    pub fn producer(&self) -> RingBufferProducer {
        RingBufferProducer {
            queue: Arc::clone(&self.queue),
            dropped_count: Arc::clone(&self.dropped_count),
        }
    }

    /// 컨슈머 핸들 생성 (처리 스레드용)
    pub fn consumer(&self) -> RingBufferConsumer {
        RingBufferConsumer {
            queue: Arc::clone(&self.queue),
            dropped_count: Arc::clone(&self.dropped_count),
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// 프로듀서 — 캡처 스레드에서 사용
pub struct RingBufferProducer {
    queue: Arc<ArrayQueue<Vec<f32>>>,
    dropped_count: Arc<AtomicU64>,
}

impl RingBufferProducer {
    /// 청크 추가. 버퍼가 가득 찬 경우 드롭하고 경고 로그.
    pub fn push(&self, samples: Vec<f32>) {
        if self.queue.push(samples).is_err() {
            let count = self.dropped_count.fetch_add(1, Ordering::Relaxed) + 1;
            if count % 100 == 1 {
                warn!("Audio ring buffer overflow: dropped {} chunks total", count);
            }
        }
    }
}

/// 컨슈머 — 처리 스레드에서 사용
pub struct RingBufferConsumer {
    queue: Arc<ArrayQueue<Vec<f32>>>,
    dropped_count: Arc<AtomicU64>,
}

impl RingBufferConsumer {
    /// 청크 하나 꺼내기 (없으면 None)
    pub fn pop(&self) -> Option<Vec<f32>> {
        self.queue.pop()
    }

    /// 사용 가능한 모든 청크를 하나의 Vec으로 합치기
    pub fn drain(&self) -> Vec<f32> {
        let mut result = Vec::new();
        while let Some(chunk) = self.queue.pop() {
            result.extend(chunk);
        }
        result
    }

    /// 현재 대기 중인 청크 수
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// 누적 드롭 횟수
    pub fn dropped_count(&self) -> u64 {
        self.dropped_count.load(Ordering::Relaxed)
    }
}

// Send + Sync — 스레드 간 안전하게 전달 가능
unsafe impl Send for RingBufferProducer {}
unsafe impl Sync for RingBufferProducer {}
unsafe impl Send for RingBufferConsumer {}
unsafe impl Sync for RingBufferConsumer {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let buf = AudioRingBuffer::new(4);
        let producer = buf.producer();
        let consumer = buf.consumer();

        producer.push(vec![1.0, 2.0]);
        producer.push(vec![3.0, 4.0]);

        assert_eq!(consumer.len(), 2);
        assert_eq!(consumer.pop().unwrap(), vec![1.0, 2.0]);
        assert_eq!(consumer.pop().unwrap(), vec![3.0, 4.0]);
        assert!(consumer.pop().is_none());
    }

    #[test]
    fn test_overflow_drops() {
        let buf = AudioRingBuffer::new(2);
        let producer = buf.producer();
        let consumer = buf.consumer();

        producer.push(vec![1.0]);
        producer.push(vec![2.0]);
        producer.push(vec![3.0]); // overflow

        assert_eq!(consumer.dropped_count(), 1);
    }

    #[test]
    fn test_drain() {
        let buf = AudioRingBuffer::new(4);
        let producer = buf.producer();
        let consumer = buf.consumer();

        producer.push(vec![1.0, 2.0]);
        producer.push(vec![3.0, 4.0]);

        let all = consumer.drain();
        assert_eq!(all, vec![1.0, 2.0, 3.0, 4.0]);
        assert!(consumer.is_empty());
    }
}
