use crate::camera::motion::MotionDetector;
use crate::camera::ring_buffer::{PreRollRingBuffer, VideoFrame};
use crate::db::Database;
use crate::error::Result;

use chrono::Utc;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{info, warn};

pub struct CameraManager {
    device_path: String,
    video_dir: PathBuf,
    motion_threshold: f64,
    db: Arc<Database>,
    recording_enabled: Arc<AtomicBool>,
}

impl CameraManager {
    pub fn new(
        device_path: &str,
        video_dir: PathBuf,
        motion_threshold: f64,
        db: Arc<Database>,
        recording_enabled: Arc<AtomicBool>,
    ) -> Self {
        Self {
            device_path: device_path.to_string(),
            video_dir,
            motion_threshold,
            db,
            recording_enabled,
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
            // Check dynamic GUI camera toggle state
            if !self.recording_enabled.load(Ordering::Relaxed) {
                if is_recording {
                    is_recording = false;
                    recording_frames.clear();
                }
                sleep(Duration::from_millis(1000)).await;
                continue;
            }

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
                data: dummy_frame_bytes.clone(),
            };

            ring_buffer.push(frame.clone());

            if detected {
                peak_motion_score = peak_motion_score.max(score);
                last_motion_instant = Some(now_instant);

                if !is_recording {
                    info!("📷 Motion Triggered (Score: {:.2})! Starting -10s circular pre-roll recording...", score);
                    is_recording = true;
                    motion_start_time = Some(now_utc);
                    recording_frames = ring_buffer.drain_pre_roll();
                }
            }

            if is_recording {
                recording_frames.push(frame);

                if let Some(last_motion) = last_motion_instant {
                    if now_instant.duration_since(last_motion) > Duration::from_secs(10) {
                        info!("📷 Motion stopped + 10s post-roll completed. Saving MP4 video clip...");

                        if let Some(start_time) = motion_start_time {
                            let duration_secs = now_utc.signed_duration_since(start_time).num_seconds() as u64;
                            let filename = format!("motion_{}.mp4", start_time.format("%Y%m%d_%H%M%S"));
                            let video_path = self.video_dir.join(&filename);

                            if let Err(e) = self.save_mp4_video(&video_path, &recording_frames) {
                                warn!("Failed to save video file {:?}: {}", video_path, e);
                            } else {
                                let start_str = start_time.to_rfc3339();
                                let end_str = now_utc.to_rfc3339();

                                match self.db.record_motion_event_and_correlate(
                                    &start_str,
                                    &end_str,
                                    peak_motion_score,
                                    &filename,
                                    &video_path.to_string_lossy(),
                                    "",
                                    duration_secs,
                                ) {
                                    Ok((_motion_id, video_id, correlated_count)) => {
                                        info!("Successfully saved video clip '{}' (ID: {}) with {} correlated BLE devices!", filename, video_id, correlated_count);
                                    }
                                    Err(e) => warn!("Database motion event logging error: {}", e),
                                }
                            }
                        }

                        is_recording = false;
                        recording_frames.clear();
                        motion_start_time = None;
                        last_motion_instant = None;
                        peak_motion_score = 0.0;
                    }
                }
            }

            sleep(Duration::from_millis(200)).await;
        }
    }

    fn generate_frame_bytes(&self, frame: u64, is_rec: bool) -> Vec<u8> {
        let mut data = vec![0u8; 1000];
        let offset = ((frame * 13) % 255) as u8;
        for i in 0..data.len() {
            data[i] = ((i as u8) ^ offset).wrapping_add(if is_rec { 50 } else { 0 });
        }
        data
    }

    fn save_mp4_video(&self, path: &PathBuf, frames: &[VideoFrame]) -> Result<()> {
        let mut file = File::create(path)?;
        file.write_all(b"FTYP_MP42_SIMULATED_VIDEO_STREAM\n")?;
        for frame in frames {
            file.write_all(&frame.data[..100.min(frame.data.len())])?;
        }
        file.flush()?;
        Ok(())
    }
}
