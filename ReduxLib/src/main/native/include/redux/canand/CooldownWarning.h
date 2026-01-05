// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <units/time.h>
#include <vector>
#include <string>
#include <frc/Timer.h>
namespace redux::canand {
/**
 * Class that yells at the user if feed is called too often in too short a succession.
 * 
 * Typically used to help prevent obliterating flash.
 */
class CooldownWarning {
  
  public:
    /**
     * Alias for std::vector's size_type.
     */
    using size_type = std::vector<units::second_t>::size_type;
    /**
     * Constructor.
     * @param threshold Maximum number of seconds that need to pass between the first and last calls
     * @param cnt Number of calls that must pass within thresholdSeconds to trigger the warning
     */
    inline CooldownWarning(units::second_t threshold, size_type cnt) : threshold(threshold), sz(cnt) {
        for (size_type i = 0; i < sz; i++) {
            count.push_back(0_ms);
        }
    };

    /**
     * Feed the CooldownWarning.
     * @return whether the error should trigger
     */
    inline bool feed() {
        if (latch) return false;
        units::second_t now = frc::Timer::GetFPGATimestamp();
        count[idx] = now;
        idx = (idx + 1) % sz;
        units::second_t past = count[idx];
        if ((now - past) < threshold) {
            return true;
        }
        return false;
    }
  private:
    units::second_t threshold;
    std::vector<units::second_t> count;
    size_type sz;
    size_type idx{0};
    bool latch{false};
};
}