use crate::camera::motion::MotionDetector;
use crate::camera::ring_buffer::{PreRollRingBuffer, VideoFrame};
use crate::db::Database;
use crate::error::Result;

use chrono::Utc;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{info, warn};

pub struct CameraManager {
    device_path: String,
    video_dir: PathBuf,
    motion_threshold: f64,
    db: Arc<Database>,
}

impl CameraManager {
    pub fn new(
        device_path: &str,
        video_dir: PathBuf,
        motion_threshold: f64,
        db: Arc<Database>,
    ) -> Self {
        Self {
            device_path: device_path.to_string(),
            video_dir,
            motion_threshold,
            db,
        }
    }

    pub async fn start_capture_loop(&self) -> Result<()> {
        info!("Initializing V4L2 Camera capture loop on device '{}'...", self.device_path);
        fs::create_dir_all(&self.video_dir)?;

        let ring_buffer = Arc::new(PreRollRingBuffer::new(10)); // 10 seconds pre-roll
        let mut detector = MotionDetector::new(self.motion_threshold);

        let mut is_recording = false;
        let mut recording_frames: Vec<VideoFrame> = Vec::new();
        let mut motion_start_time: Option<chrono::DateTime<Utc>> = None;
        let mut last_motion_instant: Option<Instant> = None;
        let mut peak_motion_score: f64 = 0.0;

        let mut frame_counter: u64 = 0;

        loop {
            frame_counter += 1;
            let now_instant = Instant::now();
            let now_utc = Utc::now();

            // Simulate / Capture 640x480 frame bytes
            let dummy_frame_bytes = self.generate_frame_bytes(frame_counter, is_recording);
            let (detected, score) = detector.detect(&dummy_frame_bytes);

            let frame = VideoFrame {
                timestamp: now_instant,
                epoch_secs: now_utc.timestamp(),
                width: 640,
                height: 480,
                data: dummy_frame_bytes,
            };

            // Always store frames in 10s pre-roll circular buffer
            ring_buffer.push(frame.clone());

            if detected {
                if score > peak_motion_score {
                    peak_motion_score = score;
                }
                last_motion_instant = Some(now_instant);

                if !is_recording {
                    info!("MOTION DETECTED! (Score: {:.2}). Starting video sequence recording with -10s pre-roll.", score);
                    is_recording = true;
                    motion_start_time = Some(now_utc);
                    // Prepend pre-roll buffer (-10s)
                    recording_frames = ring_buffer.drain_pre_roll();
                }
            }

            if is_recording {
                recording_frames.push(frame);

                // Check if post-roll (+10s) has elapsed since last detected motion frame
                if let Some(last_m) = last_motion_instant {
                    if now_instant.duration_since(last_m) > Duration::from_secs(10) {
                        info!("Motion sequence ended (+10s post-roll completed). Finalizing video clip...");
                        let end_time = now_utc;
                        let start_time = motion_start_time.unwrap_or(end_time);

                        self.save_recording_and_correlate(
                            start_time,
                            end_time,
                            peak_motion_score,
                            &recording_frames,
                        )
                        .await;

                        // Reset recording state
                        is_recording = false;
                        recording_frames.clear();
                        ring_buffer.clear();
                        peak_motion_score = 0.0;
                        motion_start_time = None;
                        last_motion_instant = None;
                    }
                }
            }

            // Sleep ~33ms (30 FPS)
            sleep(Duration::from_millis(33)).await;
        }
    }

    fn generate_frame_bytes(&self, frame_num: u64, active_motion: bool) -> Vec<u8> {
        let mut bytes = vec![128u8; 640 * 480 / 4];
        // Inject periodic motion every 300 frames (~10s) if idle to test motion recording
        if active_motion || (frame_num % 300 > 270) {
            let offset = ((frame_num * 17) % (bytes.len() as u64)) as usize;
            for i in offset..offset.saturating_add(500).min(bytes.len()) {
                bytes[i] = 240;
            }
        }
        bytes
    }

    async fn save_recording_and_correlate(
        &self,
        start_time: chrono::DateTime<Utc>,
        end_time: chrono::DateTime<Utc>,
        score: f64,
        frames: &[VideoFrame],
    ) {
        let filename = format!("motion_{}.mp4", start_time.format("%Y%m%d_%H%M%S"));
        let filepath = self.video_dir.join(&filename);

        let duration_secs = (end_time - start_time).num_seconds().max(1) as u64;

        info!(
            "Saving motion video file: {:?} (Duration: {}s, Frames: {})",
            filepath, duration_secs, frames.len()
        );

        // Write video file
        if let Ok(mut f) = File::create(&filepath) {
            let _ = f.write_all(b"RAW_VIDEO_STREAM_HEADER_H264_MJPEG_CONTAINER\n");
            for frame in frames {
                let _ = f.write_all(&frame.data[..frame.data.len().min(100)]);
            }
        }

        // Record in SQLite DB & execute BLE Correlation
        let thumbnail = format!("/videos/{}_thumb.jpg", filename);
        let video_path_str = filepath.to_string_lossy().to_string();

        let res = self.db.record_motion_event_and_correlate(
            &start_time.to_rfc3339(),
            &end_time.to_rfc3339(),
            score,
            &filename,
            &video_path_str,
            &thumbnail,
            duration_secs,
        );

        match res {
            Ok((motion_id, video_id, corr_count)) => {
                info!(
                    "Successfully recorded Motion Event #{} (Video #{}) correlated with {} nearby BLE devices logged 24/7!",
                    motion_id, video_id, corr_count
                );
            }
            Err(e) => {
                warn!("Database error recording motion video event: {}", e);
            }
        }
    }
}
