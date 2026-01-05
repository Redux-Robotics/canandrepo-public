// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor;

import java.util.concurrent.atomic.AtomicInteger;

import com.reduxrobotics.canand.*;
import com.reduxrobotics.frames.DoubleFrame;
import com.reduxrobotics.frames.Frame;
import com.reduxrobotics.frames.FrameData;
import com.reduxrobotics.frames.LongFrame;

import edu.wpi.first.hal.HAL;
import edu.wpi.first.hal.FRCNetComm.tResourceType;

/**
 * Class for the CAN interface of the 
 * <a href="https://docs.reduxrobotics.com/canandcolor/index.html">Canandcolor.</a>
 * 
 * 
 * <p>
 * Operations that receive data from the device (proximity, color, faults, temperature) generally do
 * not block. 
 * The object receives data asynchronously from the CAN packet receive thread and reads thus return 
 * the last data received.
 * </p>
 * <p>
 * Operations that set settings or change offsets will generally wait for up to 20ms by default as 
 * they will usually wait for a confirmation packet to be received in response -- unless the 
 * blocking timeout is set to zero, in which case the operations will not block.
 * </p>
 * 
 * Example code:
 * <pre class="include-com_reduxrobotics_frames_FrameData">
 * <code class="language-java">
 * Canandcolor canandcolor = new Canandcolor(0); // device id 0 
 * 
 * // Reading the Canandcolor
 * // returns a proximity value [0..1] inclusive. Values increase as objects move away from 
 * // the sensor in an approximately linear relationship.
 * canandcolor.getProximity(); 
 * 
 * // these are all also bounded [0..1] inclusive.
 * canandcolor.getRed();
 * canandcolor.getGreen();
 * canandcolor.getBlue();
 * canandcolor.getHSVHue();
 * canandcolor.getHSVSaturation();
 * canandcolor.getHSVValue();
 * 
 * // you can also get a compound ColorData object with getColor
 * // this will also have red/green/blue and hue/sat/value getters,
 * // and is most relevant if used with frames
 * ColorData colorData = canandcolor.getColor();
 * 
 * // read digout channels over CAN
 * // see the DigoutChannel class docs for more detailed information on how these work
 * // and how to configure them.
 * canandcolor.digout1().getValue(); // regular value
 * canandcolor.digout1().getStickyValue(); // sticky value
 * canandcolor.getDigoutState(); // full digout state as a DigoutState object
 * 
 * // Changing configuration
 * CanandcolorSettings settings = new CanandcolorSettings();
 * // set the proximity frame period to be sent every 10 ms
 * settings.setProximityFramePeriod(0.010);
 * // set the color frame period to be sent every 10 ms (instead of the default)
 * settings.setColorFramePeriod(0.010);
 * // set the color sensor integration to happen over 100 milliseconds
 * settings.setColorIntegrationPeriod(ColorPeriod.k100ms); 
 * canandcolor.setSettings(settings, 0.10); // apply the new settings to the device, with maximum 
 *                                           // 20 ms timeout per settings op
 * 
 * // Faults
 * canandcolor.clearStickyFaults(); // clears all sticky faults (including the power cycle flag). 
 *                                  // This call does not block.
 * 
 * // The power cycle flag will always be true on boot until the sticky faults have been cleared, 
 * // so if this is true the device has rebooted sometime between clearStickyFaults and now.
 * CanandcolorFaults faults = canandcolor.getStickyFaults(); // fetches faults
 * System.out.printf("Canandcolor rebooted: %b\n", faults.powerCycle());
 * 
 * // Timestamped data
 * // gets current proximity + timestamp together
 * FrameData&lt;Double&gt; distFrame = canandcolor.getProximityFrame().getFrameData(); 
 * distFrame.getValue(); // fetched proximity value
 * distFrame.getTimestamp(); // timestamp of the proximity reading
 * </code>
 * </pre>
 */
public class Canandcolor extends CanandDevice {


    // internal state
    /** internal Frame variable holding current proximity state */
    protected final DoubleFrame<Double> proximity = new DoubleFrame<Double>(0.0, 0, 0.0, (double v) -> v);

    /** internal Frame variable holding current color state */
    protected final ColorFrame color = new ColorFrame(0, new ColorData(0, 0, 0));

    /** internal Frame variable holding current digital output state */
    protected final LongFrame<DigoutSlotState> digout = new LongFrame<DigoutSlotState>(0, 0, new DigoutSlotState(), DigoutSlotState::new);
    /** internal Frame variable holding current status value state */
    protected final LongFrame<CanandcolorStatus> status = new LongFrame<CanandcolorStatus>(0, 0, CanandcolorStatus.invalid(), CanandcolorStatus::fromLong);


    private final CanandAddress addr;
    private final CanandSettingsManager<CanandcolorSettings> stg;
    private final DigoutChannel digout1;
    private final DigoutChannel digout2;
    private static AtomicInteger reportingIndex = new AtomicInteger(0);

    /**
     * Instantiates a new Canandcolor object. 
     * 
     * This object will be constant with respect to whatever CAN id assigned to it, so if a device 
     * changes id it may change which device this object reads from.
     * @param devID the device id to use [0..63]
     */
    public Canandcolor(int devID) {
        this(devID, "halcan");
    }

    /**
     * Instantiates a new Canandcolor with a given bus string.
     * @param devID the device id assigned to it.
     * @param bus a bus string.
     */
    public Canandcolor(int devID, String bus) {
        super();
        // 6 is a "ultrasonic sensor" -- of course neither onboard sensor IC is ultrasonic 
        // (but has similar refresh rates to one ;w;)
        // the product ID is 0
        addr = new CanandAddress(bus, 6, devID);
        stg = new CanandSettingsManager<>(this, CanandcolorSettings::new);
        digout1 = new DigoutChannel(this, DigoutChannel.Index.kDigout1);
        digout2 = new DigoutChannel(this, DigoutChannel.Index.kDigout2);

        HAL.report(tResourceType.kResourceType_Redux_future2, reportingIndex.incrementAndGet());
    }

    /**
     * Gets the currently sensed proximity normalized between [0..1] inclusive.
     * <p> The value decreases as an object gets closer to the sensor.</p>
     * 
     * <p>
     * Note that proximity is not given a unit as different materials and sensor configurations can 
     * greatly vary how the proximity value translates to actual real-world units.
     * It is generally presumed that users will have to finetune specific thresholds for 
     * applications anyway and units may not be meaningful or accurate.
     * </p>
     * 
     * @return proximity value (range [0..1] inclusive)
     */
    public double getProximity() {
        return proximity.getData();
    }

    /**
     * Red intensity, normalized [0..1] inclusive where 0 is none and 1 is as bright as possible.
     * 
     * @return red intensity [0..1]
     */
    public double getRed() {
        return color.getRed();
    }

    /**
     * Blue intensity, normalized [0..1] inclusive where 0 is none and 1 is as bright as possible.
     * 
     * @return green intensity [0..1]
     */
    public double getGreen() {
        return color.getGreen();
    }
    /**
     * Blue intensity, normalized [0..1] inclusive where 0 is none and 1 is as bright as possible.
     * 
     * @return blue intensity [0..1]
     */
    public double getBlue() {
        return color.getBlue();
    }

    /**
     * HSV colorspace hue, normalized [0 inclusive ..1.0 exclusive)
     * 
     * <p>This can be used to more accurately determine the color of a sensed object.</p>
     * @return hue [0..1)
     */
    public double getHSVHue() {
        return color.getHSVHue();
    }

    /**
     * HSV colorspace saturation, normalized [0..1] inclusive.
     * @return saturation [0..1]
     */
    public double getHSVSaturation() {
        return color.getHSVSaturation();
    }

    /**
     * HSV colorspace value, normalized [0..1] inclusive.
     * 
     * <p>This is the maximum value out of the red/green/blue values.</p>
     * @return value [0..1]
     */
    public double getHSVValue() {
        return color.getHSVValue();
    }

    /**
     * Returns a ColorData object which can also convert to the HSV colorspace.
     * 
     * @return color object
     * @see ColorData
     */
    public ColorData getColor() {
        return color.getValue();
    }

    /**
     * Returns a DigoutSlotState object representing the current state of the digital outputs as reported over CAN.
     * @return digital output state object
     */
    public DigoutSlotState getDigoutState() {
        return digout.getValue();
    }

    /**
     * Returns sticky faults.
     * Sticky faults are the active faults, except once set they do not become unset until 
     * {@link #clearStickyFaults()} is called.
     * 
     * @return {@link CanandcolorFaults} of the sticky faults.
     * @see #getActiveFaults()
     */
    public CanandcolorFaults getStickyFaults() {
        return status.getValue().stickyFaults();
    }

    /**
     * Returns an object representing currently active faults.
     * Active faults are only active for as long as the error state exists.
     * 
     * @return {@link CanandcolorFaults} of the active faults
     * @see #getStickyFaults()
     */
    public CanandcolorFaults getActiveFaults() {
        return status.getValue().activeFaults();
    }

    /**
     * Clears sticky faults. 
     * 
     * <p>It is recommended to clear this during initialization, so one can check if the device
     * loses power during operation later. </p>
     * <p>This call does not block, so it may take up to the next status frame (default every 1000 
     * ms) for the sticky faults to be updated. To check for validity, use 
     * {@link CanandcolorFaults#faultsValid()} for faults returned by {@link #getStickyFaults()}</p>
     */
    public void clearStickyFaults() {
        synchronized(status) {
            sendCANMessage(CanandcolorDetails.Msg.kClearStickyFaults, 0, CanandcolorDetails.Msg.kDlcMin_ClearStickyFaults);
            // reset status framedata such that faults are now invalid again
            status.clearData();
        }
    }

    /**
     * Get onboard device temperature readings in degrees Celsius.
     * 
     * This temperature is not particularly accurate or precise, but is meant as a rough approximation.
     * @return temperature in degrees Celsius
     */
    public double getTemperature() {
        return status.getValue().temperature();
    }

    /**
     * Controls "party mode" -- an device identification tool that blinks the onboard LED
     * various colors if level != 0.
     * 
     * This function does not block.
     * 
     * @param level the party level value to set. 
     */
    public void setPartyMode(int level) {
        if (level < 0 || level > 10) {
            throw new IllegalArgumentException(
                String.format("party level %d is not between 0 and 10", level));
        }
        sendCANMessage(CanandcolorDetails.Msg.kPartyMode, level, 1);
    }

    /**
     * Fetches the device's current configuration in a blocking manner, with control over failure
     * handling.
     * 
     * <p>This method works by requesting the device first send back all settings, and then waiting
     * for up to a specified timeout for all settings to be received by the robot controller. 
     * If the timeout is zero, this step is skipped. 
     * 
     * <p>If there are settings that were not received by the timeout, then this function will 
     * attempt to individually fetched each setting for up to a specified number of attempts. 
     * If the fresh argument is true and the timeout argument is 0, then only this latter step runs,
     * which can be used to only fetch settings that are missing from the known settings cache 
     * returned by {@link #getSettingsAsync}. 
     * 
     * <p>The resulting set of known (received) settings is then returned, complete or not.
     * 
     * <p>This function blocks, so it is best to put this in init routines rather than a main loop.
     * 
     * <pre>
     * Canandcolor color = new Canandcolor(0); 
     * 
     * // Typical usage
     * // fetch all settings with a timeout of 320 ms, and retry missing values 3 times
     * CanandcolorSettings stg = color.getSettings(0.350, 0.1, 3);
     * 
     * // Advanced usage
     * color.startFetchSettings(); // send a "fetch settings command"
     * 
     * // wait some amount of time
     * stg = color.getSettingsAsync();
     * stg.allSettingsReceived(); // may or may not be true
     * 
     * stg = color.getSettings(0, 0.1, 3); // only fetch the missing settings
     * stg.allSettingsReceived(); // far more likely to be true
     * </pre>
     * 
     * <p> <b>Note that this function may return incomplete settings!</b> 
     * Use {@link CanandcolorSettings#allSettingsReceived()} to verify all settings were received.
     * 
     * @param timeout maximum number of seconds to wait for settings before giving up.
     * @param missingTimeout maximum number of seconds to wait for each settings retry before giving up.
     * @param attempts number of attempts to try and fetch values missing from the first pass
     * @return {@link CanandcolorSettings} representing the device's configuration
     */
    public CanandcolorSettings getSettings(double timeout, double missingTimeout, int attempts) {
        return stg.getSettings(timeout, missingTimeout, attempts);
    }
    /**
     * Fetches the device's current configuration in a blocking manner.
     * This function will block for up to the specified number of seconds waiting for the device to 
     * reply, so it is best to put this in a teleop or autonomous init function, rather than the 
     * main loop.
     * 
     * <p>If settings time out, it will retry each missing setting once with a 20ms timeout, and if
     *  they still fail, a partial Settings will still be returned.
     * 
     * <p> <b>Note that this function may return incomplete settings!</b> 
     * Use {@link CanandcolorSettings#allSettingsReceived()} to verify all settings were received.
     * 
     * @param timeout maximum number of seconds to wait for settings before giving up
     * @return {@link CanandcolorSettings} representing the device's configuration
     */
    public CanandcolorSettings getSettings(double timeout) {
        return stg.getSettings(timeout, 0.1, 1);
    }

    /**
     * Fetches the Canandcolor's current configuration in a blocking manner.
     * This function will block for up to 0.350 seconds waiting for the device to reply, so it is 
     * best to put this in an init function rather than the main loop.
     * 
     * <p> <b>Note that this function may return incomplete settings!</b> 
     * Use {@link CanandcolorSettings#allSettingsReceived()} to verify all settings were received.
     * @return {@link CanandcolorSettings} representing the device's configuration
     */
    public CanandcolorSettings getSettings() {
        return getSettings(0.350);
    }

    /**
     * Sets the brightness of the onboard lamp LED.
     * 
     * <p><b>This value does not persist on device reboot!</b> 
     * Use {@link CanandcolorSettings#setLampLEDBrightness(double)} and {@link #setSettings(CanandcolorSettings)} to set this persistently.
     * <p>
     * The LED can also be physically turned off regardless of setting with the onboard switch.
     * </p>
     * By factory default this setting is set to max brightness (1.0)
     * @param brightness scaled brightness from 0.0 (off) to 1.0 (max brightness)
     */
    public void setLampLEDBrightness(double brightness) {
        if (brightness < 0 || brightness > 1) {
            throw new IllegalArgumentException("brightness must be between 0 and 1 inclusive");
        }
        stg.setSettingById(CanandcolorDetails.Stg.kLampBrightness, (long) (brightness * 36000), CanandSettingsManager.kFlag_Ephemeral);
    }

    /**
     * Tells the Canandcolor to begin transmitting its settings; once they are all transmitted 
     * (after ~200-300ms), the values can be retrieved from {@link Canandcolor#getSettingsAsync()}
     */
    public void startFetchSettings() {
        stg.startFetchSettings();
    }

    /**
     * Non-blockingly returns a {@link CanandcolorSettings} object of the most recent known 
     * settings values received from the device.
     *
     * <p><b>Most users will probably want to use {@link Canandcolor#getSettings()} instead. </b></p>
     * 
     * One can call this after a {@link Canandcolor#startFetchSettings()} call, and use 
     * {@link CanandcolorSettings#allSettingsReceived()} to check if/when all values have been 
     * seen. As an example:
     * 
     * <pre>
     * 
     * // somewhere in an init function
     * Canandcolor canandcolor = new Canandcolor(0); 
     * canandcolor.startFetchSettings();
     * 
     * // ...
     * // somewhere in a loop function
     * 
     * if (canandcolor.getSettingsAsync().allSettingsReceived()) {
     *   // do something with the settings object
     *   System.out.printf("Canandcolor lamp brightness: %f\n",
     *     canandcolor.getSettingsAsync().getLampLEDBrightness().get());
     * }
     * </pre>
     * 
     * 
     * If this is called after {@link Canandcolor#setSettings(CanandcolorSettings)}, this method 
     * will return a settings object where only the fields where the device has echoed the new 
     * values back will be populated. To illustrate this, consider the following:
     * <pre>
     * 
     * // somewhere in initialization (just as a definition):
     * Canandcolor canandcolor = new Canandcolor(0); 
     * 
     * // somewhere in a loop 
     * canandcolor.setSettings(new CanandcolorSettings().setProximityFramePeriod(0.100));
     * 
     * // will likely return Optional.empty(), as the device hasn't confirmed the previous transaction
     * canandcolor.getSettingsAsync().getProximityFramePeriod();
     * 
     * // after up to ~300 ms...
     * canandcolor.getSettingsAsync().getProximityFramePeriod(); // will likely return 0.1 seconds
     * </pre>
     * 
     * @see Canandcolor#startFetchSettings
     * @return CanandcolorSettings object of known settings
     */
    public CanandcolorSettings getSettingsAsync() {
        return stg.getKnownSettings();
    }

    /**
     * Applies the settings from a {@link CanandcolorSettings} object to the device, with fine
     * grained control over failure-handling.
     * 
     * This overload allows specifiyng the number of retries per setting as well as the confirmation
     * timeout. Additionally, it returns a {@link CanandcolorSettings} object of settings that 
     * were not able to be successfully applied.
     * 
     * @see CanandcolorSettings
     * @param settings the {@link CanandcolorSettings} to update the encoder with
     * @param timeout maximum time in seconds to wait for each setting to be confirmed. Set to 0 to 
     *     not check (and not block).
     * @param attempts the maximum number of attempts to write each individual setting
     * @return a CanandcolorSettings object of unsuccessfully set settings.
     */
    public CanandcolorSettings setSettings(CanandcolorSettings settings, double timeout, int attempts) {
        return stg.setSettings(settings, timeout, attempts);
    }

    /**
     * Applies the settings from a {@link CanandcolorSettings} object to the device. 
     * For more information, see the {@link CanandcolorSettings} class documentation.
     * @see CanandcolorSettings
     * @param settings the {@link CanandcolorSettings} to update the device with
     * @param timeout maximum time in seconds to wait for each setting to be confirmed. Set to 0 to
     *     not check (and not block).
     * @return true if successful, false if a setting operation timed out
     */
    public boolean setSettings(CanandcolorSettings settings, double timeout) {
        return stg.setSettings(settings, timeout);
    }

    /**
     * Applies the settings from a {@link CanandcolorSettings} object to the device. 
     * For more information, see the {@link CanandcolorSettings} class documentation.
     * @see CanandcolorSettings
     * @param settings the {@link CanandcolorSettings} to update the device with
     * @return true if successful, false if a setting operation timed out
     */
    public boolean setSettings(CanandcolorSettings settings) {
        return setSettings(settings, 0.10);
    }

    /**
     * Clears all sticky digout flags, as readable by {@link DigoutSlotState#getStickyDigoutValue(DigoutChannel.Index)} 
     * from {@link #getDigoutState()}
     */
    public void clearStickyDigoutFlags() {
        sendCANMessage(CanandcolorDetails.Msg.kClearStickyDigout, 0, 0);
    }

    /**
     * Resets the Canandcolor to factory defaults. 
     * @param timeout how long to wait for the new settings to be confirmed by the device in seconds
     *     (suggested at least 0.35 seconds)
     * @return {@link CanandcolorSettings} object of received settings. 
     *     Use {@link CanandcolorSettings#allSettingsReceived()} to verify success.
     */
    public CanandcolorSettings resetFactoryDefaults(double timeout) {
        return stg.sendReceiveSettingCommand(
            CanandcolorDetails.Enums.SettingCommand.kResetFactoryDefault,
            timeout,
            true
        );
    }
    /**
     * Resets the device to factory defaults, waiting up to 350 ms to confirm the settings changes.
     * @return {@link CanandcolorSettings} object of received settings. 
     *     Use {@link CanandcolorSettings#allSettingsReceived()} to verify success.
     */
    public CanandcolorSettings resetFactoryDefaults() {
        return resetFactoryDefaults(0.350);
    }

    /**
     * Returns the first digout channel.
     * @return digout channel 1
     */
    public DigoutChannel digout1() {
        return digout1;
    }

    /**
     * Returns the second digout channel.
     * @return digout channel 2
     */
    public DigoutChannel digout2() {
        return digout2;
    }

    /**
     * Returns the proximity reading frame.
     * @return the proximity reading frame, which will hold the current proximity reading.
     * @see Frame
     */
    public DoubleFrame<Double> getProximityFrame() {
        return proximity;
    }

    /**
     * Returns the color reading frame, which includes CAN timestamp data.
     * @return the color reading frame, which will hold timestamped color readings
     * @see Frame
     */
    public ColorFrame getColorFrame() {
        return color;
    }

    /**
     * Returns the digital output state frame, which includes CAN timestamp data.
     * @return the digital output state frame 
     */
    public LongFrame<DigoutSlotState> getDigoutFrame() {
        return digout;
    }

    /**
     * Returns the current status frame, which includes CAN timestamp data.
     * {@link FrameData} objects are immutable.
     * @return the current status frame, as a {@link CanandcolorStatus} record.
     */
    public LongFrame<CanandcolorStatus> getStatusFrame() {
        return status;
    }

    /**
     * Returns the {@link CanandSettingsManager} associated with this device.
     * 
     * The {@link CanandSettingsManager} is an internal helper object. 
     * Teams are typically not expected to use it except for advanced cases (e.g. custom settings
     * wrappers)
     * @return internal settings manager handle
     */
    public CanandSettingsManager<CanandcolorSettings> getInternalSettingsManager() {
        return stg;
    }


    /* various behind the scenes stuff */
    @Override
    public void handleMessage(CanandMessage msg) {
        // This method takes incoming messages and updates the device's recorded state accordingly.
        //byte[] data = msg.getData();
        double timestamp = msg.getTimestamp();

        switch(msg.getApiIndex()) {
            case CanandcolorDetails.Msg.kDistanceOutput: {
                if (msg.getLength() != CanandcolorDetails.Msg.kDlc_DistanceOutput) break;
                proximity.updateData(CanandcolorDetails.Msg.extractDistanceOutput_Distance(msg.getDataAsLong()) / 65535.0, timestamp);
                break;
            }
            case CanandcolorDetails.Msg.kColorOutput: {
                if (msg.getLength() != CanandcolorDetails.Msg.kDlc_ColorOutput) break;
                color.updateData(msg.getDataAsLong(), timestamp);
                break;
            }
            case CanandcolorDetails.Msg.kDigitalOutput: {
                if (msg.getLength() != CanandcolorDetails.Msg.kDlc_DigitalOutput) break;
                digout.updateData(msg.getDataAsLong(), timestamp);
            }
            case CanandcolorDetails.Msg.kStatus: {
                if (msg.getLength() != CanandcolorDetails.Msg.kDlc_Status) break;
                status.updateData(msg.getDataAsLong(), timestamp);
                break;
            }
            case CanandcolorDetails.Msg.kReportSetting: {
                if (stg == null) break;
                stg.handleSetting(msg);
                break;
            }
            default:
            break;
        }
    }

    @Override
    public CanandAddress getAddress() {
        return addr;
    }

    @Override
    public CanandFirmwareVersion getMinimumFirmwareVersion() {
        return new CanandFirmwareVersion(2024, 0, 0);
    }
}