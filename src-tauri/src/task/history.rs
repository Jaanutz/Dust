use std::collections::VecDeque;

use tokio::time::Instant;

pub struct DownloadHistory {
    pub bytes: VecDeque<(Instant, u64)>,
    pub last_push: Instant,
}

impl DownloadHistory {
    const MAX_ENTRIES: usize = 16;

    pub fn new() -> Self {
        DownloadHistory {
            bytes: VecDeque::new(),
            last_push: Instant::now(),
        }
    }

    fn can_push(&self) -> bool {
        self.last_push.elapsed().as_millis() >= 200
    }

    pub fn try_push(&mut self, bytes: u64) {
        if !self.can_push() {
            return;
        }

        self.bytes.push_back((Instant::now(), bytes));
        if self.bytes.len() > Self::MAX_ENTRIES {
            self.bytes.pop_front();
        }
        self.last_push = Instant::now();
    }

    pub fn clear(&mut self) {
        self.bytes.clear();
    }
}

impl DownloadHistory {
    pub fn average_speed(&self) -> Option<f64> {
        if let (Some((start_time, start_bytes)), Some((end_time, end_bytes))) =
            (self.bytes.front(), self.bytes.back())
        {
            let duration = end_time.duration_since(*start_time).as_secs_f64();
            if duration > 0.1 {
                return Some((end_bytes - start_bytes) as f64 / duration as f64);
            }
        }
        None
    }
}
