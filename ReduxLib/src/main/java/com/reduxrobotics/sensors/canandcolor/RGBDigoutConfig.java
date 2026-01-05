// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor;

/**
 * A digital output configuration that uses RGB thresholds.
 * <p>
 * Basic usage example:
 * </p>
 * <pre>
 * Canandcolor color = new Canandcolor(0);
 * // Check if the an object is close and the color sensor is seeing red for at least 10 milliseconds,
 * color.digout1().configureSlots(new RGBDigoutConfig()
 *   .setMaxProximity(0.25)
 *   .setProximityInRangeFor(0.01)
 *   .setMinRed(0.1)
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
 * @see DigoutChannel#configureSlots(DigoutConfig)
 */
public final class RGBDigoutConfig extends DigoutConfig {
    /**
     * Default constructor.
     */
    public RGBDigoutConfig() {}
    /**
     * Sets the value that the proximity reading has to be greater than or equal to for the digout channel to be true.
     * <p>If not specified, this value is 0.0</p>
     * @param thresh proximity threshold
     * @return calling object
     */
    public RGBDigoutConfig setMinProximity(double thresh) { 
        this.proximityLow = thresh;
        return this; 
    }

    /**
     * Sets the value that the proximity reading has to be less than or equal to for the digout channel to be true.
     * <p>If not specified, this value is 1.0</p>
     * @param thresh proximity threshold
     * @return calling object
     */
    public RGBDigoutConfig setMaxProximity(double thresh) { 
        this.proximityHigh = thresh;
        return this; 
    }

    /**
     * Sets the minimum time that the proximity reading has to match the upper and lower thresholds 
     * in order for the digout channel to be true.
     * <p>This can be used to "debounce" readings by requiring the proximity to match the thresholds for multiple readings.</p>
     * 
     * <p>This threshold has millisecond resolution, but the units are still seconds.</p>
     * @param seconds The minimum number of seconds. The default value is 0 (zero time required)
     * @return calling object
     */
    public RGBDigoutConfig setProximityInRangeFor(double seconds) {
        this.proximityDebounce = seconds;
        return this;
    }

    /**
     * Sets the value that the red color reading has to be greater than or equal to for the digout channel to be true.
     * <p>If not specified, this value is 0.0</p>
     * @param thresh color value threshold
     * @return calling object
     */
    public RGBDigoutConfig setMinRed(double thresh) { 
        this.redOrHueLow = thresh;
        return this; 
    }

    /**
     * Sets the value that the red color reading has to be greater than or equal to for the digout channel to be true.
     * <p>If not specified, this value is 1.0</p>
     * @param thresh color value threshold
     * @return calling object
     */
    public RGBDigoutConfig setMaxRed(double thresh) { 
        this.redOrHueHigh = thresh;
        return this; 
    }

    /**
     * Sets the value that the green color reading has to be greater than or equal to for the digout channel to be true.
     * <p>If not specified, this value is 0.0</p>
     * @param thresh color value threshold
     * @return calling object
     */
    public RGBDigoutConfig setMinGreen(double thresh) { 
        this.greenOrSatLow = thresh;
        return this; 
    }

    /**
     * Sets the value that the green color reading has to be greater than or equal to for the digout channel to be true.
     * <p>If not specified, this value is 1.0</p>
     * @param thresh color value threshold
     * @return calling object
     */
    public RGBDigoutConfig setMaxGreen(double thresh) { 
        this.greenOrSatHigh = thresh;
        return this; 
    }

    /**
     * Sets the value that the blue color reading has to be greater than or equal to for the digout channel to be true.
     * <p>If not specified, this value is 0.0</p>
     * @param thresh color value threshold
     * @return calling object
     */
    public RGBDigoutConfig setMinBlue(double thresh) { 
        this.blueOrValueLow = thresh;
        return this; 
    }

    /**
     * Sets the value that the blue color reading has to be greater than or equal to for the digout channel to be true.
     * <p>If not specified, this value is 1.0</p>
     * @param thresh color value threshold
     * @return calling object
     */
    public RGBDigoutConfig setMaxBlue(double thresh) { 
        this.blueOrValueHigh = thresh;
        return this; 
    }

    /**
     * Sets the minimum time that the color readings have to match the upper and lower RGB thresholds 
     * in order for the digout channel to be true.
     * <p>This can be used to "debounce" readings by requiring the color to match the thresholds for multiple readings.</p>
     * 
     * <p>This threshold has millisecond resolution, but the units are still seconds.</p>
     * @param seconds The minimum number of seconds. The default value is 0 (zero time required)
     * @return calling object
     */
    public RGBDigoutConfig setColorInRangeFor(double seconds) {
        this.proximityDebounce = seconds;
        return this;
    }
    
    @Override
    boolean isHSV() {
        return false;
    }
    
}
