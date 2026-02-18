use std::collections::VecDeque;

use tokio::time::Instant;

pub struct DownloadHistory {
    pub bytes: VecDeque<(Instant, u64)>,
}

impl DownloadHistory {
    const MAX_ENTRIES: usize = 16;
    const MIN_MILLIS_BETWEEN_PUSHES: u128 = 200;

    pub fn new() -> Self {
        DownloadHistory {
            bytes: VecDeque::new(),
        }
    }

    fn can_push(&self) -> bool {
        self.bytes
            .back()
            .map(|(time, _)| time.elapsed().as_millis() >= Self::MIN_MILLIS_BETWEEN_PUSHES)
            .unwrap_or(true)
    }

    pub fn try_push(&mut self, bytes: u64) {
        if !self.can_push() {
            return;
        }

        self.bytes.push_back((Instant::now(), bytes));
        if self.bytes.len() > Self::MAX_ENTRIES {
            self.bytes.pop_front();
        }
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
