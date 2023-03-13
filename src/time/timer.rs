use std::time::Duration;

pub struct Timer {
    duration: Duration,
    repeating: bool,
    
    iteration: u32,
    finished: bool,
    elapsed_time: Duration,
}

impl Timer {
    pub fn init(duration: Duration, repeating: bool) -> Self {
        Timer {
            duration,
            repeating,
            
            iteration: 1,
            finished: false,
            elapsed_time: Duration::ZERO,
        }
    }
    
    pub fn iteration(&self) -> u32 {
        self.iteration.clone()
    }
    
    pub fn finished(&self) -> bool {
        self.finished.clone()
    }
    
    pub fn tick(&mut self, step: Duration){
        if self.repeating {
            self.elapsed_time += step;
            
            if self.elapsed_time >= self.duration {
                self.elapsed_time -= self.duration;
                self.iteration += 1;
            }
        } else if !self.finished {
            self.elapsed_time += step;
            
            if self.elapsed_time >= self.duration {
                self.finished = true;
                self.elapsed_time = self.duration;
            }
        }
    }
}
