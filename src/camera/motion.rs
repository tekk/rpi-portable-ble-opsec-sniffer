pub struct MotionDetector {
    prev_frame: Option<Vec<u8>>,
    threshold: f64,
}

impl MotionDetector {
    pub fn new(threshold: f64) -> Self {
        Self {
            prev_frame: None,
            threshold,
        }
    }

    /// Calculates motion score between current frame and previous frame.
    /// Returns (is_motion_detected, motion_score).
    pub fn detect(&mut self, current_frame_bytes: &[u8]) -> (bool, f64) {
        if self.prev_frame.is_none() {
            self.prev_frame = Some(current_frame_bytes.to_vec());
            return (false, 0.0);
        }

        let prev = self.prev_frame.as_ref().unwrap();
        let min_len = prev.len().min(current_frame_bytes.len());

        if min_len == 0 {
            return (false, 0.0);
        }

        let mut diff_sum: u64 = 0;
        // Sample every 16th byte for fast motion check on RPi CPU
        let step = 16;
        let mut samples = 0;

        for i in (0..min_len).step_by(step) {
            let d = (prev[i] as i16 - current_frame_bytes[i] as i16).abs() as u64;
            if d > 25 {
                diff_sum += d;
            }
            samples += 1;
        }

        let score = if samples > 0 {
            (diff_sum as f64) / (samples as f64)
        } else {
            0.0
        };

        self.prev_frame = Some(current_frame_bytes.to_vec());

        let detected = score >= self.threshold;
        (detected, score)
    }
}
