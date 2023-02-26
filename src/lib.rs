
use std::{collections::HashMap, time::{SystemTime, UNIX_EPOCH}};
pub struct RateLimiter {
    rate_limit: u32,
    counts: HashMap<String, u32>,
    reset_time: u64,
}

impl RateLimiter {
    pub fn new(
      rate_limit: u32,
    ) -> Self {
        let reset_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            / 60
            * 60;
        Self {
            rate_limit: rate_limit,
            counts: HashMap::new(),
            reset_time,
        }
    }

    pub fn check_rate_limit(&mut self, ip: &str) -> bool {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            / 60
            * 60;
        if current_time > self.reset_time {
            self.counts.clear();
            self.reset_time = current_time;
        }
        println!("RateLimit: {}: {}", ip, self.counts.get(ip).unwrap_or(&0));
        let count = self.counts.entry(ip.to_owned()).or_insert(0);
        if *count >= self.rate_limit {
            false
        } else {
            *count += 1;
            true
        }
    }
}
