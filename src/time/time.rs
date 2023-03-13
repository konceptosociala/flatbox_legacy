use std::time::{Instant, Duration};

pub struct Time {
    startup_time: Instant,
    latest_update: Option<Instant>,
    delta_time: Duration,
}

impl Time {
    pub fn new() -> Self {
        Time {
            startup_time: Instant::now(),
            latest_update: None,
            delta_time: Duration::ZERO,
        }
    }
    
    pub fn delta_time(&self) -> Duration {
        self.delta_time.clone()
    }
    
    pub fn update(&mut self){
        let now = Instant::now();
        let delta = now - self.latest_update.unwrap_or(self.startup_time);
        
        self.latest_update = Some(now);
        self.delta_time = delta;
    }
}
