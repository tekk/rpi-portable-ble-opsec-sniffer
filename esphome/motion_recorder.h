#pragma once

#include "esphome.h"
#include <vector>

class Esp32MotionRecorder : public Component {
public:
    float motion_score = 0.0;
    bool motion_active = false;
    uint32_t motion_events_count = 0;

    void setup() override {
        ESP_LOGI("MotionRecorder", "Initialized ESP32 Camera / Sensor Motion Engine");
    }

    void process_frame_diff(int current_diff) {
        motion_score = (float)current_diff / 10.0f;
        if (motion_score > 15.0f) {
            if (!motion_active) {
                motion_active = true;
                motion_events_count++;
                ESP_LOGI("MotionRecorder", "MOTION DETECTED on ESP32! Score: %.2f", motion_score);
            }
        } else {
            motion_active = false;
        }
    }
};
