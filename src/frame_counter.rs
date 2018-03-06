
use std::time::Instant;

pub struct FrameCounter {
    start: Instant,
    frames: u64,
}

impl FrameCounter {
    pub fn new() -> FrameCounter {
        FrameCounter {
            start: Instant::now(),
            frames: 0,
        }
    }


    pub fn tick(&mut self) -> Option<f64> {
        self.frames += 1;
        let now = Instant::now();
        let duration = now - self.start;

        let secs = duration.as_secs() as f64 + 1e-9 * duration.subsec_nanos() as f64;
        if secs > 0.5 {
            let fps = self.frames as f64 / secs;
            self.frames = 0;
            self.start = now;
            Some(fps)
        } else {
            None
        }
    }
}


