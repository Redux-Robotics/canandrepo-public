// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.canand;

import edu.wpi.first.wpilibj.DriverStation;

/**
 * Class that yells at the user if {@link #feed} is called too often in too short a succession.
 * 
 * Typically used to help prevent obliterating flash.
 */
public class CooldownWarning {
    private double count[];
    private final String warning;
    private final double thresholdSeconds;
    private int idx = 0;
    private boolean latch = false;

    /**
     * Constructor.
     * 
     * @param warning The warning to report to the driverstation
     * @param thresholdSeconds Maximum number of seconds that need to pass between the first and last calls
     * @param thresholdCount Number of calls that must pass within thresholdSeconds to trigger the warning
     */
    public CooldownWarning(String warning, double thresholdSeconds, int thresholdCount) {
        this.warning = warning;
        this.thresholdSeconds = thresholdSeconds;
        count = new double[thresholdCount];
        for (int i = 0; i < count.length; i++) {
            count[i] = 0; 
        }
    }

    /**
     * Feed the CooldownError.
     */
    public synchronized void feed() {
        if (latch) return;
        double now = CanandUtils.getFPGATimestamp();
        count[idx] = now;
        idx = (idx + 1) % count.length;
        double past = count[idx];
        if ((now - past) < thresholdSeconds) {
            DriverStation.reportError(warning, true);
            latch = true;
        }
    }
    
}
