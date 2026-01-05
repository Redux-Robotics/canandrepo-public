// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandmag;
import java.util.concurrent.atomic.AtomicInteger;

import com.reduxrobotics.canand.*;
import com.reduxrobotics.frames.ByteArrayFrame;
import com.reduxrobotics.frames.DoubleFrame;
import com.reduxrobotics.frames.Frame;
import com.reduxrobotics.frames.FrameData;

import edu.wpi.first.hal.HAL;
import edu.wpi.first.hal.FRCNetComm.tResourceType;

/**
 * Class for the CAN interface of the 
 * <a href="https://docs.reduxrobotics.com/canandmag/index.html">Canandmag.</a>
 * 
 * <p>
 * If you are using a Canandmag with Spark Max or Talon with the PWM output, see 
 * <a href="https://docs.reduxrobotics.com/canandmag/spark-max.html">our Spark Max docs</a>
 * or
 * <a href="https://docs.reduxrobotics.com/canandmag/talon-srx.html">our Talon SRX docs</a>
 * for information on how to use the encoder with the Rev and CTRE APIs.
 * </p>
 * 
 * <p>
 * In general, the Java API will use SI units (seconds, meters, deg Celsius), with the
 * exception of rotation being expressed in turns (+1 rotation == 1.0) 
 * </p>
 * 
 * <p>
 * Operations that receive data from the device (position, velocity, faults, temperature) generally 
 * do not block. 
 * The object receives data asynchronously from the CAN packet receive thread and reads thus return 
 * the last data received.
 * </p>
 * <p>
 * Operations that set settings or change offsets will generally wait for up to 20ms by default as 
 * they will by default wait for a confirmation packet to be received in response -- unless the 
 * blocking timeout is set to zero, in which case the operation swill not block.
 * </p>
 * 
 * Example code:
 * <pre class="include-com_reduxrobotics_frames_FrameData">
 * Canandmag canandmag = new Canandmag(0); // encoder id 0 
 * 
 * // Reading the Canandmag
 * canandmag.getPosition(); // returns a multi-turn relative position, in rotations (turns)
 * canandmag.getAbsPosition(); // returns an absolute position bounded from 0 inclusive to 1 
 *                               // exclusive over one rotation
 * canandmag.getVelocity(); // returns measured velocity in rotations per second
 * 
 * // Updating position
 * canandmag.setPosition(-3.5); // sets the relative position to -3.5 turns with default 
 *                                // confirmation timeout of 20 ms (does not persist on reboot)
 * canandmag.setAbsPosition(0.330, 0); // sets the absolute position to 0.5 turns without blocking
 *                                       // for confirmation (persists on reboot)
 * canandmag.zeroAll(); // sets both the relative and absolute position to zero
 * 
 * // Changing configuration
 * CanandmagSettings settings = new CanandmagSettings();
 * settings.setVelocityFilterWidth(25); // sets the velocity filter averaging period to 25 ms
 * settings.setInvertDirection(true); // make positive be clockwise instead of ccw opposite the 
 *                                    // sensor face
 * settings.setPositionFramePeriod(0.010); // set the position frame period to be sent every 10 ms
 * canandmag.setSettings(settings, 0.10); // apply the new settings to the device, with maximum 
 *                                           // 20 ms timeout per settings operation
 * 
 * // Faults
 * canandmag.clearStickyFaults(); // clears all sticky faults (including the power cycle flag). 
 *                                  // This call does not block.
 * 
 * // this flag will always be true on boot until the sticky faults have been cleared, 
 * // so if this is true the encoder has rebooted sometime between clearStickyFaults and now.
 * CanandmagFaults faults = canandmag.getStickyFaults(); // fetches faults
 * System.out.printf("Encoder rebooted: %d\n", faults.powerCycle());
 * 
 * // Timestamped data
 * // gets current position + timestamp together
 * FrameData&lt;Double&gt; posFrameData = canandmag.getPositionFrame().getFrameData(); 
 * posFrameData.getValue(); // fetched position in rotations
 * posFrameData.getTimestamp(); // timestamp of the previous position
 * </pre>
 */
public class Canandmag extends CanandDevice {

    // internal state
    /** internal Frame variable holding current relative position state */
    protected DoubleFrame<Double> position = new DoubleFrame<Double>(0.0, 0.0, 0.0, (double v) -> v);

    /** internal Frame variable holding current absolute position state */
    protected DoubleFrame<Double> absPosition = new DoubleFrame<Double>(0.0, 0.0, 0.0, (double v) -> v);

    /** internal Frame variable holding current velocity state */
    protected DoubleFrame<Double> velocity = new DoubleFrame<Double>(0.0, 0.0, 0.0, (double v) -> v);

    /** internal Frame variable holding current status value state */
    protected ByteArrayFrame<CanandmagStatus> status = new ByteArrayFrame<CanandmagStatus>(4, 0.0, CanandmagStatus.invalid(), CanandmagStatus::fromByteArray);


    private CanandAddress addr;
    //private Settings knownSettings = new CanandmagSettings();
    private CanandSettingsManager<CanandmagSettings> stg;


    /** Conversion factor from velocity packet ticks per second to rotations per second. */
    public static final double kCountsPerRotationPerSecond = 1024;

    /** Conversion factor for number of position packet ticks per rotation. */
    public static final double kCountsPerRotation = 16384;
    private static AtomicInteger reportingIndex = new AtomicInteger(0);

    /**
     * Instantiates a new Canandmag object. 
     * 
     * This object will be constant with respect to whatever CAN id assigned to it, so if a device 
     * changes id it may change which device this object reads from.
     * @param devID the device id to use [0..63]
     */
    public Canandmag(int devID) {
        this(devID, "halcan");
    }

    /**
     * Instantiates a new Canandmag with a given bus string.
     * @param devID the device id assigned to it.
     * @param bus a bus string.
     */
    public Canandmag(int devID, String bus) {
        super();
        // 7 is a "gear tooth sensor"
        // the product ID is 0
        addr = new CanandAddress(bus, 7, devID);
        stg = new CanandSettingsManager<>(this, CanandmagSettings::new);
        HAL.report(tResourceType.kResourceType_Redux_future1, reportingIndex.incrementAndGet());
    }

    /**
     * Gets the current integrated relative position in rotations. 
     * 
     * <p> This value does not wrap around, so turning a sensed axle multiple rotations will return 
     * multiple sensed rotations of position. 
     * 
     * <p> By default, positive is in the counter-clockwise direction from the sensor face, and also
     * counter-clockwise looking at the LED side of the throughbore.
     * 
     * <p> On encoder power-on, unlike the absolute value, this value will always initialize to zero.
     * @return signed relative position in rotations (range [-131072.0..131071.999938396484])
     */
    public double getPosition() {
        return position.getData();
    }

    /**
     * Gets the current absolute position of the encoder, scaled from 0 inclusive to 1 exclusive.
     * 
     * <p> By default, higher values are in the counter-clockwise direction from the sensor face, 
     * and also counter-clockwise looking at the LED side of the throughbore.
     * <p> This value will persist across encoder power cycles making it appropriate for swerves/arms/etc. </p>
     * @return absolute position in fraction of a rotation [0..1)
     */
    public double getAbsPosition() {
        return absPosition.getData();
    }

    /**
     * Sets the new relative (multi-turn) position of the encoder to the given value.
     * 
     * <p>
     * Note that this does not update the absolute position, and this value is lost on a power cycle.
     * To update the absolute position, use {@link #setAbsPosition(double, double)}
     * </p>
     * 
     * @param newPosition new relative position in rotations (acceptable range [-131072.0..131071.99993896484])
     * @param timeout maximum time in seconds to wait for a setting to be confirmed. Set to 0 to not
     *      check (and not block).
     * @return true on success, false on setting failure
     */
    public boolean setPosition(double newPosition, double timeout) {
        if (newPosition < -131072 || newPosition >= 131072) 
            throw new IllegalArgumentException(String.format("new relative position %f is not in the range [-131072..131072)", newPosition)); 
        long newPos = (long) (newPosition * kCountsPerRotation);
        return stg.confirmSetSetting(CanandmagDetails.kStg_RelativePosition, newPos, timeout, 0).isValid();
    }

    /**
     * Sets the new relative (multi-turn) position of the encoder to the given value, with a 
     * confirmation timeout of 20 ms.
     * 
     * <p>
     * Note that this does not update the absolute position, and this value is lost on power cycle. 
     * To update the absolute position, use {@link #setAbsPosition(double)}
     * </p>
     * 
     * @param newPosition new relative position in rotations (acceptable range [-131072.0..131071.99993896484])
     * @return true on success, false on setting failure
     */
    public boolean setPosition(double newPosition) {
        return setPosition(newPosition, 0.10);
    }

    private CooldownWarning setAbsPositionWarning = new CooldownWarning(
        "Calling setAbsPosition() at high frequency will quickly wear out the Canandmag's internal flash.\n" + 
        "Consider either using setPosition() instead or passing in ephemeral=true to not write to flash.",
        1, 5);

    /**
     * Sets the new absolute position value for the encoder which will (by default) persist across reboots.
     * 
     * @param newPosition new absolute position in fraction of a rotation (acceptable range [0..1))
     * @param timeout maximum time in seconds to wait for the operation to be confirmed. Set to 0 to
     *     not check (and not block).
     * @param ephemeral if true, set the setting ephemerally
     * @return true on success, false on setting failure
     */
    public boolean setAbsPosition(double newPosition, double timeout, boolean ephemeral) {
        if (newPosition < 0 || newPosition >= 1) { 
            throw new IllegalArgumentException(
                String.format("new absolute position %f is not in the range [0..1)", newPosition));
        }
        long newPos = (long) (newPosition * kCountsPerRotation);
        // the 1 in byte 3 specifies to set the zero offset so the read out position will be the new
        // specified position
        int flags = CanandSettingsManager.kFlag_Ephemeral;
        if (!ephemeral) {
            flags = 0;
            setAbsPositionWarning.feed();
        }
        return stg.confirmSetSetting(CanandmagDetails.kStg_ZeroOffset, (long) (newPos & 0x3fff | 0x10000), timeout, flags).isValid();
    }

    /**
     * Sets the new absolute position value for the encoder which will persist across reboots with 
     * a specified timeout.
     * 
     * @param newPosition new absolute position in fraction of a rotation (acceptable range [0..1))
     * @param timeout maximum time in seconds to wait for the operation to be confirmed. Set to 0 to
     *     not check (and not block).
     * @return true on success, false on timeout
     */
    public boolean setAbsPosition(double newPosition, double timeout) {
        return setAbsPosition(newPosition, timeout, false);
    }

    /**
     * Sets the new absolute position value for the encoder which will persist across reboots with 
     * default timeout of 20 ms.
     * 
     * @param newPosition new absolute position in fraction of a rotation (acceptable range [0..1))
     * @return true on success, false on timeout
     */
    public boolean setAbsPosition(double newPosition) {
        return setAbsPosition(newPosition, 0.10, false);
    }

    /**
     * Sets both the current absolute and relative encoder position to 0 -- generally equivalent to 
     * pressing the physical zeroing button on the encoder.
     * @param timeout maximum time in seconds to wait for each operation (zeroing absolute and 
     *     relative position) to be confirmed. Set to 0 to not check (and not block).
     * @return true on success, false on timeout
     */
    public boolean zeroAll(double timeout) {
        return setPosition(0, timeout) && setAbsPosition(0, timeout, false);
    }
    /**
     * Sets both the current absolute and relative encoder position to 0 -- generally equivalent to 
     * pressing the physical zeroing button on the encoder.
     * 
     * <p>This will wait up to 20 ms to each of the absolute and relative positions, so up to 100 ms
     * total (realistically less)</p>
     * @return true on success, false on timeout
     */
    public boolean zeroAll() {
        return zeroAll(0.10);
    }

    /**
     * Returns the measured velocity in rotations per second.
     * @return velocity, in rotations (turns) per second
     */
    public double getVelocity() {
        return velocity.getData();
    }


    /**
     * Returns whether the encoder magnet is in range of the sensor or not.
     * This can be seen visually on the sensor -- a green LED is in range, whereas 
     * a red LED is out of range.
     * 
     * @return whether the output shaft magnet is in range.
     */
    public boolean magnetInRange()  {
        return status.getValue().magnetInRange();
    }

    /**
     * Returns sticky faults.
     * Sticky faults are the active faults, except once set they do not become unset until 
     * {@link #clearStickyFaults()} is called.
     * 
     * @return {@link CanandmagFaults} of the sticky faults.
     * @see #getActiveFaults()
     */
    public CanandmagFaults getStickyFaults() {
        return status.getValue().stickyFaults();
    }

    /**
     * Returns an object representing currently active faults.
     * Active faults are only active for as long as the error state exists.
     * 
     * @return {@link CanandmagFaults} of the active faults
     * @see #getStickyFaults()
     */
    public CanandmagFaults getActiveFaults() {
        return status.getValue().activeFaults();
    }
    /**
     * Get onboard encoder temperature readings in degrees Celsius.
     * @return temperature in degrees Celsius
     */
    public double getTemperature() {
        return status.getValue().temperature();
    }

    /**
     * Get the contents of the previous status packet, which includes active faults, sticky faults, and temperature.
     * @return device status as a {@link CanandmagStatus} record
     */
    public CanandmagStatus getStatus() {
        return status.getValue();
    }

    /**
     * Clears sticky faults. 
     * 
     * <p>It is recommended to clear this during initialization, so one can check if the encoder 
     * loses power during operation later.
     * 
     * <p>This call does not block, so it may take up to the next status frame (default every 
     * 1000 ms) for the sticky faults to be updated. To check for validity, use 
     * {@link CanandmagFaults#faultsValid()} for faults returned by {@link #getStickyFaults()}
     */
    public void clearStickyFaults() {
        synchronized(status) {
            byte data[] = {0};
            sendCANMessage(CanandmagDetails.kMsg_ClearStickyFaults, data);
            // reset status framedata such that faults are now invalid again
            status.clearData();
        }
    }

    /**
     * Controls "party mode" -- an encoder identification tool that blinks the onboard LED
     * various colors at a user-specified strobe period.
     * The strobe period of the LED will be (50 milliseconds * level). Setting this to 0 disables 
     * party mode.
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
        byte data[] = {(byte) level};
        sendCANMessage(CanandmagDetails.kMsg_PartyMode, data);
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
     * returned by {@link #getSettingsAsync()}. 
     * 
     * <p>The resulting set of known (received) settings is then returned, complete or not.
     * 
     * <p>This function blocks, so it is best to put this in init routines rather than a main loop.
     * 
     * <pre>
     * Canandmag enc = new Canandmag(0); 
     * 
     * // Typical usage
     * // fetch all settings with a timeout of 320 ms, and retry missing values 3 times
     * CanandmagSettings stg = enc.getSettings(0.350, 0.1, 3);
     * 
     * // Advanced usage
     * enc.startFetchSettings(); // send a "fetch settings command"
     * 
     * // wait some amount of time
     * stg = enc.getSettingsAsync();
     * stg.allSettingsReceived(); // may or may not be true
     * 
     * stg = enc.getSettings(0, 0.1, 3); // only fetch the missing settings, with 20ms timeout on each
     * stg.allSettingsReceived(); // far more likely to be true
     * </pre>
     * 
     * <p> <b>Note that this function may return incomplete settings!</b> 
     * Use {@link CanandmagSettings#allSettingsReceived()} to verify all settings were received.
     * 
     * @param timeout maximum number of seconds to wait for settings before giving up.
     * @param missingTimeout maximum number of seconds to wait for each settings retry before giving up.
     * @param attempts number of attempts to try and fetch values missing from the first pass
     * @return {@link CanandmagSettings} representing the device's configuration
     */
    public CanandmagSettings getSettings(double timeout, double missingTimeout, int attempts) {
        return stg.getSettings(timeout, missingTimeout, attempts);
    }

    /**
     * Fetches the device's current configuration in a blocking manner.
     * 
     * This function will block for up to the specified number of seconds waiting for the device to 
     * reply, so it is best to put this in a teleop or autonomous init function, rather than the 
     * main loop.
     * 
     * <p>If settings time out, it will retry each missing setting once with a 20ms timeout, and if
     *  they still fail, a partial Settings will still be returned.
     * 
     * <p> <b>Note that this function may return incomplete settings!</b> 
     * Use {@link CanandmagSettings#allSettingsReceived()} to verify all settings were received.
     * 
     * @param timeout maximum number of seconds to wait for settings before giving up
     * @return {@link CanandmagSettings} representing the device's configuration
     */
    public CanandmagSettings getSettings(double timeout) {
        return stg.getSettings(timeout, 0.1, 1);
    }

    /**
     * Fetches the Canandmag's current configuration in a blocking manner.
     * 
     * <p> This function will block for up to 0.350 seconds waiting for the encoder to reply, so it 
     * is best to put this in a teleop or autonomous init function, rather than the main loop.
     * 
     * <p> <b>Note that this function may return incomplete settings!</b> 
     * Use {@link CanandmagSettings#allSettingsReceived()} to verify all settings were received.
     * 
     * @return {@link CanandmagSettings} representing the device's configuration
     */
    public CanandmagSettings getSettings() {
        return getSettings(0.350);
    }

    /**
     * Tells the device to begin transmitting its settings.
     * 
     * Once they are all transmitted (after ~200-300ms), the values can be retrieved from 
     * {@link #getSettingsAsync()}
     */
    public void startFetchSettings() {
        stg.startFetchSettings();
    }

    /**
     * Non-blockingly returns a {@link CanandmagSettings} object of the most recent known 
     * settings values received from the encoder.
     *
     * <p> <b>Most users will probably want to use {@link Canandmag#getSettings()} instead. </b>
     * 
     * One can call this after a {@link Canandmag#startFetchSettings()} call, and use 
     * {@link CanandmagSettings#allSettingsReceived()} to check if/when all values have been 
     * seen. As an example:
     * 
     * <pre>
     * 
     * // somewhere in an init function
     * Canandmag enc = new Canandmag(0); 
     * enc.startFetchSettings();
     * 
     * // ...
     * // somewhere in a loop function
     * 
     * if (enc.getSettingsAsync().allSettingsReceived()) {
     *   // do something with the settings object
     *   System.out.printf("Encoder velocity frame period: %d\n", 
     *       enc.getSettingsAsync().getVelocityFramePeriod());
     * }
     * </pre>
     * 
     * 
     * If this is called after {@link Canandmag#setSettings(Canandmag.CanandmagSettings)}, this method 
     * will return a settings object where only the fields where the encoder has echoed the new 
     * values back will be populated. To illustrate this, consider the following:
     * <pre>
     * // somewhere in an init function
     * Canandmag enc = new Canandmag(0); 
     * 
     * // somewhere in a loop 
     * enc.setSettings(new CanandmagSettings().setVelocityFramePeriod(0.100));
     * 
     * // This will likely return empty, as the encoder hasn't confirmed the previous transaction
     * enc.getSettingsAsync().getVelocityFramePeriod(); 
     * 
     * // after up to 100 ms...
     * enc.getSettingsAsync().getVelocityFramePeriod(); // will likely return 100
     * </pre>
     * 
     * @see Canandmag#startFetchSettings
     * @return CanandmagSettings object of known settings
     */
    public CanandmagSettings getSettingsAsync() {
        return stg.getKnownSettings();
    }

    /**
     * Applies the settings from a {@link CanandmagSettings} object to the device, with fine
     * grained control over failure-handling.
     * 
     * This overload allows specifiyng the number of retries per setting as well as the confirmation
     * timeout. Additionally, it returns a {@link CanandmagSettings} object of settings that 
     * were not able to be successfully applied.
     * 
     * @see CanandmagSettings
     * @param settings the {@link CanandmagSettings} to update the encoder with
     * @param timeout maximum time in seconds to wait for each setting to be confirmed. Set to 0 to 
     *     not check (and not block).
     * @param attempts the maximum number of attempts to write each individual setting
     * @return a CanandmagSettings object of unsuccessfully set settings.
     */
    public CanandmagSettings setSettings(CanandmagSettings settings, double timeout, int attempts) {
        return stg.setSettings(settings, timeout, attempts);
    }

    /**
     * Applies the settings from a {@link CanandmagSettings} object to the Canandmag. 
     * For more information, see the {@link CanandmagSettings} class documentation.
     * @see CanandmagSettings
     * @param settings the {@link CanandmagSettings} to update the encoder with
     * @param timeout maximum time in seconds to wait for each setting to be confirmed. Set to 0 to 
     *     not check (and not block).
     * @return true if successful, false if a setting operation failed
     */
    public boolean setSettings(CanandmagSettings settings, double timeout) {
        return stg.setSettings(settings, timeout);
    }

    /**
     * Applies the settings from a {@link CanandmagSettings} object to the Canandmag. 
     * For more information, see the {@link CanandmagSettings} class documentation.
     * @see CanandmagSettings
     * @param settings the {@link CanandmagSettings} to update the encoder with
     * @return true if successful, false if a setting operation timed out
     */
    public boolean setSettings(CanandmagSettings settings) {
        return setSettings(settings, 0.10);
    }

    /**
     * Resets the encoder to factory defaults, and then wait for all settings to be broadcasted 
     * back.
     * @param clearZero whether to clear the zero offset from the encoder's memory as well
     * @param timeout how long to wait for the new settings to be confirmed by the encoder in 
     *     seconds (suggested at least 0.35 seconds)
     * @return {@link CanandmagSettings} object of received settings. 
     *     Use {@link CanandmagSettings#allSettingsReceived()} to verify success.
     */
    public CanandmagSettings resetFactoryDefaults(boolean clearZero, double timeout) {
        int val = (clearZero) ? CanandmagDetails.kStgCmd_ResetFactoryDefault : CanandmagDetails.kStgCmd_ResetFactoryDefaultKeepZero; 
        return stg.sendReceiveSettingCommand(val, timeout, true);
    }
    /**
     * Resets the encoder to factory defaults, waiting up to 350 ms to confirm the settings changes.
     * @param clearZero whether to clear the zero offset from the encoder's memory as well
     * @return {@link CanandmagSettings} object of received settings. 
     *     Use {@link CanandmagSettings#allSettingsReceived()} to verify success.
     */
    public CanandmagSettings resetFactoryDefaults(boolean clearZero) {
        return resetFactoryDefaults(clearZero, 0.350);
    }

    /**
     * Returns the current relative position frame.
     * @return the current position frame, which will hold the current position in the same units as
     *     {@link #getPosition()}
     */
    public Frame<Double> getPositionFrame() {
        return position;
    }

    /**
     * Returns the current absolute position frame.
     * @return the current position frame, which will hold the current position in the same units as
     *     {@link #getAbsPosition()}
     */
    public Frame<Double> getAbsPositionFrame() {
        return absPosition;
    }

    /**
     * Returns the current velocity frame, which includes CAN timestamp data.
     * @return the current velocity frame, which will hold the current velocity in the same units as
     *      {@link #getVelocity()}
     */
    public Frame<Double> getVelocityFrame() {
        return velocity;
    }

    /**
     * Returns the current status frame, which includes CAN timestamp data.
     * {@link FrameData} objects are immutable.
     * @return the current status frame, as a {@link CanandmagStatus} record.
     */
    public Frame<CanandmagStatus> getStatusFrame() {
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
    public CanandSettingsManager<CanandmagSettings> getInternalSettingsManager() {
        return stg;
    }

    // 1=data valid
    private byte[] statusBuf = {0, 0, 0, 1};

    /* various behind the scenes stuff */
    @Override
    public void handleMessage(CanandMessage msg) {
        // This takes incoming messages and updates the encoder's recorded state accordingly.
        byte[] data = msg.getData();
        //BitSet dataAsBitSet = BitSet.valueOf(data);
        double timestamp = msg.getTimestamp();
        switch(msg.getApiIndex()) {
            case CanandmagDetails.kMsg_PositionOutput: {
                if (msg.getLength() != 6) break;
                position.updateData(
                    CanandUtils.extractLong(data, 0, 32, true) / kCountsPerRotation, timestamp);
                absPosition.updateData(
                    CanandUtils.extractLong(data, 34, 48, false) / kCountsPerRotation, timestamp);
                break;
            }
            case CanandmagDetails.kMsg_VelocityOutput: {
                if (msg.getLength() != 3) break;
                velocity.updateData(
                    CanandUtils.extractLong(data, 0, 22, true) / kCountsPerRotationPerSecond, timestamp);
                break;
            }
            case CanandmagDetails.kMsg_Status: {
                if (msg.getLength() != 8) break;
                System.arraycopy(data, 0, statusBuf, 0, 3);
                status.updateData(statusBuf, timestamp);
                break;
            }
            case CanandmagDetails.kMsg_ReportSetting: {
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
        return new CanandFirmwareVersion(2024, 2, 0);
    }
}