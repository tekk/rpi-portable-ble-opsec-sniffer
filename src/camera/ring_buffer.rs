use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VideoFrame {
    pub timestamp: Instant,
    pub epoch_secs: i64,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // JPEG / Raw frame buffer
}

pub struct PreRollRingBuffer {
    max_duration: Duration,
    frames: Mutex<VecDeque<VideoFrame>>,
}

impl PreRollRingBuffer {
    pub fn new(max_duration_secs: u64) -> Self {
        Self {
            max_duration: Duration::from_secs(max_duration_secs),
            frames: Mutex::new(VecDeque::new()),
        }
    }

    pub fn push(&self, frame: VideoFrame) {
        if let Ok(mut lock) = self.frames.lock() {
            let now = frame.timestamp;
            lock.push_back(frame);

            // Evict frames older than max_duration (e.g. -10s)
            while let Some(front) = lock.front() {
                if now.duration_since(front.timestamp) > self.max_duration {
                    lock.pop_front();
                } else {
                    break;
                }
            }
        }
    }

    pub fn drain_pre_roll(&self) -> Vec<VideoFrame> {
        if let Ok(lock) = self.frames.lock() {
            lock.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    #[allow(dead_code)]
    pub fn clear(&self) {
        if let Ok(mut lock) = self.frames.lock() {
            lock.clear();
        }
    }
}
