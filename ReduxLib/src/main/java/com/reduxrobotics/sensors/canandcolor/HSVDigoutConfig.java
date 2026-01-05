// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor;

/**
 * A digital output configuration that uses thresholds in the HSV color space to match color readings.
 * <p>
 * Basic usage example:
 * </p>
 * <pre>
 * Canandcolor color = new Canandcolor(0);
 * // Check if the an object is close and the color sensor is seeing blue for at least 10 milliseconds,
 * color.digout1().configureSlots(new HSVDigoutConfig()
 *   .setMaxProximity(0.15)
 *   .setProximityInRangeFor(0.01)
 *   .setMinHue(0.5)
 *   .setMaxHue(0.7)
 *   .setColorInRangeFor(0.01)
 * );
 * // Instantly send CAN digout frames when the above condition changes at all.
 * color.digout1().configureFrameTrigger(DigoutFrameTrigger.kRisingAndFalling);
 * // Configure the DIG-1 GPIO pin to also output the condition
 * color.digout1().configureOutputPin(DigoutPinConfig.kDigoutLogicActiveHigh);
 * // Check value over CAN
 * color.digout1().getValue();
 * </pre>
 * 
 * <p>If the minHue is larger than the maxHue, then
 * <code> (minHue &lt;= hue &lt;= 1.0) || (0.0 &lt;= hue &lt;= maxHue)</code>
 * is instead evaluated, which can be used to screen for red hues. E.g. 
 * <pre>
 * Canandcolor color = new Canandcolor(0);
 * // Check for red values:
 * color.digout1().configureSlots(new HSVDigoutConfig()
 *   .setMinHue(0.8)
 *   .setMaxHue(0.2)
 *   .setColorInRangeFor(0.01)
 * );
 * </pre>
 * 
 * <p>
 * will check for hue values from [0.8..1.0] and [0.0..0.2], allowing for proper handling
 * of the wraparound.
 * </p>
 * @see DigoutChannel#configureSlots(DigoutConfig)
 */
public final class HSVDigoutConfig extends DigoutConfig {
    /**
     * Default constructor.
     */
    public HSVDigoutConfig() { }
    /**
     * Sets the value that the proximity reading has to be greater than or equal to for the digout to be true.
     * <p>If not specified, this value is 0.0</p>
     * @param thresh proximity threshold
     * @return calling object
     */
    public HSVDigoutConfig setMinProximity(double thresh) { 
        this.proximityLow = thresh;
        return this; 
    }

    /**
     * Sets the value that the proximity reading has to be less than or equal to for the digout to be true.
     * <p>If not specified, this value is 1.0</p>
     * @param thresh proximity threshold
     * @return calling object
     */
    public HSVDigoutConfig setMaxProximity(double thresh) { 
        this.proximityHigh = thresh;
        return this; 
    }

    /**
     * Sets the minimum time that the proximity reading has to match the upper and lower thresholds 
     * in order for the digout to be true.
     * <p>This threshold has millisecond resolution, but the units are still seconds.</p>
     * 
     * <p>This threshold has millisecond resolution, but the units are still seconds.</p>
     * @param seconds The minimum number of seconds. The default value is 0 (zero time required)
     * @return calling object
     */
    public HSVDigoutConfig setProximityInRangeFor(double seconds) {
        this.proximityDebounce = seconds;
        return this;
    }

    /**
     * Sets the value that the HSV hue reading has to be greater than or equal to for the digout to be true.
     * <p>If not specified, this value is 0.0</p>
     * <p> If the min hue threshold is larger than the max hue threshold, the sensor will evaluate</p>
     * <pre class="not-code">
     * minHue &lt;= hue OR hue &lt;= maxHue 
     * </pre>
     * instead of the usual
     * <pre class="not-code">
     * minHue &lt;= hue AND hue &lt;= maxHue 
     * </pre>
     * 
     * <p>which is useful when trying to detect red objects as one could set the min hue to 0.9 and the 
     * max hue to 0.1 but still get correct results across the wraparound.</p>
     * @param thresh color value threshold
     * @return calling object
     */
    public HSVDigoutConfig setMinHue(double thresh) { 
        this.redOrHueLow = thresh;
        return this; 
    }

    /**
     * Sets the value that the HSV hue reading has to be greater than or equal to for the digout to be true.
     * <p>If not specified, this value is 1.0</p>
     * <p> If the min hue threshold is larger than the max hue threshold, the sensor will evaluate</p>
     * <pre class="not-code">
     * minHue &lt;= hue OR hue &lt;= maxHue 
     * </pre>
     * instead of the usual
     * <pre class="not-code">
     * minHue &lt;= hue AND hue &lt;= maxHue 
     * </pre>
     * 
     * <p>which is useful when trying to detect red objects as one could set the min hue to 0.9 and the 
     * max hue to 0.1 but still get correct results across the wraparound.</p>
     * @param thresh color value threshold
     * @return calling object
     */
    public HSVDigoutConfig setMaxHue(double thresh) { 
        this.redOrHueHigh = thresh;
        return this; 
    }

    /**
     * Sets the value that the HSV saturation reading has to be greater than or equal to for the digout to be true.
     * <p>If not specified, this value is 0.0</p>
     * @param thresh color value threshold
     * @return calling object
     */
    public HSVDigoutConfig setMinSaturation(double thresh) { 
        this.greenOrSatLow = thresh;
        return this; 
    }

    /**
     * Sets the value that the HSV saturation reading has to be greater than or equal to for the digout to be true.
     * <p>If not specified, this value is 1.0</p>
     * @param thresh color value threshold
     * @return calling object
     */
    public HSVDigoutConfig setMaxSaturation(double thresh) { 
        this.greenOrSatHigh = thresh;
        return this; 
    }

    /**
     * Sets the value that the HSV value reading has to be greater than or equal to for the digout to be true.
     * <p>If not specified, this value is 0.0</p>
     * @param thresh color value threshold
     * @return calling object
     */
    public HSVDigoutConfig setMinValue(double thresh) { 
        this.redOrHueLow = thresh;
        return this; 
    }

    /**
     * Sets the value that the HSV value reading has to be greater than or equal to for the digout to be true.
     * <p>If not specified, this value is 1.0</p>
     * @param thresh color value threshold
     * @return calling object
     */
    public HSVDigoutConfig setMaxValue(double thresh) { 
        this.redOrHueHigh = thresh;
        return this; 
    }

    /**
     * Sets the minimum time that the color readings have to match the upper and lower HSV thresholds 
     * in order for the digout to be true.
     * <p>This can be used to "debounce" readings by requiring the color to match the thresholds for multiple readings.</p>
     * 
     * <p>This threshold has millisecond resolution, but the units are still seconds.</p>
     * @param seconds The minimum number of seconds. The default value is 0 (zero time required)
     * @return calling object
     */
    public HSVDigoutConfig setColorInRangeFor(double seconds) {
        this.proximityDebounce = seconds;
        return this;
    }
    
    @Override
    boolean isHSV() {
        return true;
    }
    
}
