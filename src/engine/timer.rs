use std::time;

pub struct Timer {
    duration: time::Duration,
    previous_time: time::Instant,
}

impl Timer {
    /// Create timer instance with fixed duration as a second unit.
    pub fn from_second(tick_second: f64) -> Self {
        Timer {
            duration: time::Duration::from_secs_f64(tick_second),
            previous_time: time::Instant::now()
        }
    }

    /// Tick timer and update variables, return true if ticked.
    /// Otherwise, return false.
    pub fn tick(&mut self) -> bool {
        let now_time = time::Instant::now();
        let elapsed = now_time.duration_since(self.previous_time);
        if elapsed < self.duration {
            false
        }
        else {
            self.previous_time = now_time;
            true
        }
    }
}
