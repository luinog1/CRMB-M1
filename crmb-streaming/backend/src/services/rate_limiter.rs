use std::collections::VecDeque;
use tokio::time::{Duration, Instant, sleep};

#[derive(Debug)]
pub struct RateLimiter {
    max_requests: usize,
    window_duration: Duration,
    requests: VecDeque<Instant>,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_duration: Duration) -> Self {
        Self {
            max_requests,
            window_duration,
            requests: VecDeque::new(),
        }
    }
    
    pub async fn acquire(&mut self) {
        let now = Instant::now();
        
        // Remove old requests outside the window
        while let Some(&front) = self.requests.front() {
            if now.duration_since(front) > self.window_duration {
                self.requests.pop_front();
            } else {
                break;
            }
        }
        
        // If we're at the limit, wait until we can make another request
        if self.requests.len() >= self.max_requests {
            if let Some(&oldest) = self.requests.front() {
                let wait_time = self.window_duration - now.duration_since(oldest);
                if wait_time > Duration::from_millis(0) {
                    tracing::debug!("Rate limiting: waiting {:?}", wait_time);
                    sleep(wait_time).await;
                }
            }
        }
        
        // Add the current request
        self.requests.push_back(now);
        
        // Clean up again after potential sleep
        let now = Instant::now();
        while let Some(&front) = self.requests.front() {
            if now.duration_since(front) > self.window_duration {
                self.requests.pop_front();
            } else {
                break;
            }
        }
    }
    
    pub fn remaining_capacity(&self) -> usize {
        self.max_requests.saturating_sub(self.requests.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};
    
    #[tokio::test]
    async fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2, Duration::from_millis(100));
        
        // First two requests should go through immediately
        let start = Instant::now();
        limiter.acquire().await;
        limiter.acquire().await;
        assert!(start.elapsed() < Duration::from_millis(10));
        
        // Third request should be delayed
        let start = Instant::now();
        limiter.acquire().await;
        assert!(start.elapsed() >= Duration::from_millis(90));
    }
}