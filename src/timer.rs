use std::time::{Duration, Instant};

const PERIOD: Duration = Duration::from_millis(16);

pub struct Timers {
    pub delay_timer: u8,
    pub sound_timer: u8,
    last_tick: Instant,
}

impl Timers {
    pub fn new() -> Self {
        Timers {
            delay_timer: 0,
            sound_timer: 0,
            last_tick: Instant::now(),
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_tick) < PERIOD {
            return;
        }
        self.last_tick = now;

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
}
