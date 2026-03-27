use std::sync::Mutex;
use serde::{Deserialize, Serialize};

/// API 사용량 모니터
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageStats {
    pub provider: String,
    pub total_tokens: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub request_count: u64,
    pub estimated_cost_usd: f64,
}

pub struct UsageMonitor {
    stats: Mutex<Vec<UsageStats>>,
}

impl UsageMonitor {
    pub fn new() -> Self {
        Self {
            stats: Mutex::new(Vec::new()),
        }
    }

    pub fn record_usage(
        &self,
        provider: &str,
        prompt_tokens: u64,
        completion_tokens: u64,
        cost_per_1k_tokens: f64,
    ) {
        let mut stats = self.stats.lock().unwrap();
        let entry = stats.iter_mut().find(|s| s.provider == provider);

        let total = prompt_tokens + completion_tokens;
        let cost = total as f64 / 1000.0 * cost_per_1k_tokens;

        if let Some(entry) = entry {
            entry.total_tokens += total;
            entry.prompt_tokens += prompt_tokens;
            entry.completion_tokens += completion_tokens;
            entry.request_count += 1;
            entry.estimated_cost_usd += cost;
        } else {
            stats.push(UsageStats {
                provider: provider.to_string(),
                total_tokens: total,
                prompt_tokens,
                completion_tokens,
                request_count: 1,
                estimated_cost_usd: cost,
            });
        }
    }

    pub fn get_stats(&self) -> Vec<UsageStats> {
        self.stats.lock().unwrap().clone()
    }

    pub fn reset(&self) {
        self.stats.lock().unwrap().clear();
    }
}
