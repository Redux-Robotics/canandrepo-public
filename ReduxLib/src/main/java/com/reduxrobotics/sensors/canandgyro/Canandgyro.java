// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandgyro;

import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;

import com.reduxrobotics.canand.CanandAddress;
import com.reduxrobotics.canand.CanandDevice;
import com.reduxrobotics.canand.CanandMessage;
import com.reduxrobotics.canand.CanandSettingsManager;
import com.reduxrobotics.frames.ByteArrayFrame;
import com.reduxrobotics.frames.DoubleFrame;
import com.reduxrobotics.frames.Frame;

import edu.wpi.first.hal.HAL;
import edu.wpi.first.hal.FRCNetComm.tResourceType;
import edu.wpi.first.math.VecBuilder;
import edu.wpi.first.math.geometry.Quaternion;
import edu.wpi.first.math.geometry.Rotation2d;
import edu.wpi.first.math.geometry.Rotation3d;

/**
 * Class for the CAN interface of the 
 * <a href="https://docs.reduxrobotics.com/canandgyro/index.html">Canandgyro.</a>
 * 
 * <p>
 * In general, the Java API will use SI units (seconds, meters, deg Celsius), with the
 * exception of rotation being expressed in turns (+1 rotation == 1.0) 
 * </p>
 * 
 * <p>
 * Operations that receive data from the device (heading, velocity, faults, temperature) generally 
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
 * Canandgyro canandgyro = new Canandgyro(0); // gyro id 0 
 * 
 * // Reading angular position
 * canandgyro.getYaw(); // gets the yaw (Z-axis) value in rotations [-0.5 inclusive..0.5 exclusive)
 *                      // This is probably what you want to use for robot heading.
 * canandgyro.getMultiturnYaw(); // also gets yaw, except without a wraparound
 * canandgyro.getPitch(); // pitch (Y-axis) value
 * canandgyro.getRoll(); // roll (X-axis) value
 * canandgyro.getRotation2d(); // Z-axis Rotation2d object
 * canandgyro.getRotation3d(); // Full 3d rotation object
 * canandgyro.getQuaternion(); // Raw rotation quaternion object
 *                             // getQuaternion{X, Y, Z, W}() methods also exist to avoid allocation.
 * 
 * 
 * // Reading angular velocity (all in rotations per second)
 * canandgyro.getAngularVelocityYaw();
 * canandgyro.getAngularVelocityPitch();
 * canandgyro.getAngularVelocityRoll();
 * 
 * // Linear acceleration (gravitational units)
 * canandgyro.getAccelerationX();
 * canandgyro.getAccelerationY();
 * canandgyro.getAccelerationZ();
 * 
 * // Updating pose:
 * canandgyro.setYaw(0.25); // set yaw to 0.25 rotations positive
 * canandgyro.setPose(0.0, 0.1, 0.25, 0.1); // set roll, pitch, yaw as 0.0, 0.1, and 0.25 rotations
 *                                           // with 20 ms timeout
 * 
 * // Manually calibrating:
 * // The Canandgyro automatically calibrates on boot, but you may want to force a calibration.
 * // Calibration takes several seconds!!!
 * 
 * canandgyro.startCalibration(); // begin calibration
 * canandgyro.isCalibrating(); // check if the gyro is still calibrating
 * canandgyro.waitForCalibrationToFinish(5.0); // wait up to 5 seconds for calibration to finish.
 * 
 * 
 * // Changing configuration and adjusting frame periods
 * CanandgyroSettings settings = new CanandgyroSettings();
 * settings.setYawFramePeriod(20); // sets the yaw frame period to one packet every 20 ms
 * settings.setAngularPositionFramePeriod(10); // sets the angular position frame period to once 
 *                                             // every 10 ms (may be useful for balancing)
 * settings.setAccelerationFramePeriod(0); // disable accel frame periods (default quite low anyway)
 * canandgyro.setSettings(settings, 0.10); // apply the new settings to the device, with maximum 
 *                                          // 20 ms timeout per settings operation
 * 
 * // Faults
 * canandgyro.clearStickyFaults(); // clears all sticky faults (including the power cycle flag). 
 *                                  // This call does not block.
 * 
 * // this flag will always be true on boot until the sticky faults have been cleared, 
 * // so if this is true the gyro has rebooted sometime between clearStickyFaults and now.
 * CanandgyroFaults faults = canandgyro.getStickyFaults(); // fetches faults
 * System.out.printf("Device rebooted: %d\n", faults.powerCycle());
 * 
 * // Timestamped data
 * // gets current angular position + timestamp together
 * var quatFrameData = canandgyro.getAngularPositionFrame();
 * quatFrameData.getValue(); // fetched quaternion object
 * quatFrameData.getW(); // fetched quaternion W component
 * quatFrameData.getTimestamp(); // timestamp of the quaternion data
 * </pre>
 */
public class Canandgyro extends CanandDevice {
    private static final double TAU = Math.PI * 2;

    /** Yaw frame (units: rotations) */
    protected final DoubleFrame<Double> singleYaw = new DoubleFrame<Double>(0.0, 0.0, 0.0, (double v) -> v);
    /** Yaw frame (units: rotations) */
    protected final DoubleFrame<Double> multiYaw = new DoubleFrame<Double>(0.0, 0.0, 0.0, (double v) -> v);
    /** Quaternion frame */
    protected final QuaternionFrame quat = new QuaternionFrame(0.0, new Quaternion());
    /** Angular velocity frame (rotations/second) */
    protected final Vec3Frame vel = new Vec3Frame(0.0, VecBuilder.fill(0, 0, 0), 2000.0 / 360.0 / 32767.0); // deg/s -> rot/s
    /** Linear acceleration frame (gravitational unit Gs) */
    protected final Vec3Frame accel = new Vec3Frame(0.0, VecBuilder.fill(0, 0, 0), 16.0 / 32768.0); // gs
    /** Status frame */
    protected final ByteArrayFrame<CanandgyroStatus> status = new ByteArrayFrame<CanandgyroStatus>(8, 0.0, CanandgyroStatus.invalid(), CanandgyroStatus::fromByteArray);
    /** Calibrating state */
    protected final AtomicBoolean calibrating = new AtomicBoolean(false);

    private final CanandAddress addr;
    private final CanandSettingsManager<CanandgyroSettings> stg;
    private boolean useYawAngleFrame = true;
    private static AtomicInteger reportingIndex = new AtomicInteger(0);

    /**
     * Instantiates a new Canandgyro.
     * @param devID the device id assigned to it.
     */
    public Canandgyro(int devID) {
        this(devID, "halcan");
    }

    /**
     * Instantiates a new Canandgyro with a given bus string.
     * @param devID the device id assigned to it.
     * @param bus a bus string.
     */
    public Canandgyro(int devID, String bus) {
        super();
        addr = new CanandAddress(bus, 4, devID);
        stg = new CanandSettingsManager<>(this, CanandgyroSettings::new);
        HAL.report(tResourceType.kResourceType_Redux_future3, reportingIndex.incrementAndGet());
    }

    // wpilib helper objects
    /**
     * Gets a quaternion object of the gyro's 3d rotation from the zero point -- 
     * <b> Warning: this allocates objects! Limit the number of calls you make to this per robot loop!</b>
     * @return a {@link Quaternion} of the current Canandgyro pose
     */
    public Quaternion getQuaternion() {
        return quat.getValue();
    }

    /**
     * Gets a {@link Rotation3d} object of the gyro's 3d rotation from the zero point -- 
     * <b> Warning: this allocates objects! Limit the number of calls you make to this per robot loop!</b>
     * If you just want Z-axis rotation as a double, it's more performant to use {@link #getYaw()}.
     * 
     * @return a {@link Rotation3d} of the current Canandgyro pose
     */
    public Rotation3d getRotation3d() {
        return new Rotation3d(quat.getValue());
    }

    /**
     * Gets a Rotation2d object representing the rotation around the yaw axis from the zero point -- 
     * <b> Warning: this allocates objects! Limit the number of calls you make to this per robot loop!</b>
     * If you just want Z-axis rotation as a double, it's more performant to use {@link #getYaw()}.
     * @return a {@link Rotation2d} of the current Canandgyro yaw
     */
    public Rotation2d getRotation2d() {
        return new Rotation2d(singleYaw.getData() * TAU);
    }

    /**
     * Gets the W term of the current Canandgyro rotation quaternion, normalized from [-1.0..1.0] 
     * inclusive.
     * @return quaternion term value
     */
    public double getQuaternionW() {
        return quat.getW();
    }

    /**
     * Gets the X term of the current Canandgyro rotation quaternion, normalized from [-1.0..1.0] 
     * inclusive.
     * @return quaternion term value
     */
    public double getQuaternionX() {
        return quat.getX();
    }

    /**
     * Gets the Y term of the current Canandgyro rotation quaternion, normalized from [-1.0..1.0] 
     * inclusive.
     * @return quaternion term value
     */
    public double getQuaternionY() {
        return quat.getY();
    }

    /**
     * Gets the Z term of the current Canandgyro rotation quaternion, normalized from [-1.0..1.0] 
     * inclusive.
     * @return quaternion term value
     */
    public double getQuaternionZ() {
        return quat.getZ();
    }

    /**
     * Sets whether this object should use the dedicated yaw message for yaw angle instead of 
     * deriving it from the pose quaternion frame.
     * 
     * By default this is true, as the yaw angle frame is more precise and by default more frequent.
     * 
     * @param use use the yaw angle
     */
    public void useDedicatedYawAngleFrame(boolean use) {
        useYawAngleFrame = use;
    }

    /**
     * Gets the yaw (Z-axis) rotation from [-0.5 inclusive..0.5 exclusive).
     * 
     * This is probably the function you want to use for applications like field-centric control, although 
     * some libraries may want the value from {@link #getRotation2d()} instead.
     * <p>Multiplying by <code>(2 * Math.PI)</code> will give you radians, while 
     * multiplying by 360 will give you degrees. </p>
     * 
     * <p>If you want a multi-turn yaw that does not wrap around, consider {@link #getMultiturnYaw()}
     * 
     * @return yaw in rotations.
     */
    public double getYaw() {
        if (useYawAngleFrame && singleYaw.hasData()) return singleYaw.getData();
        return quat.getYaw();
    }

    /**
     * Gets a multi-turn yaw (Z-axis) rotation that tracks to multiple continuous rotations.
     * 
     * Note that this relies on the dedicated multi-turn yaw packet so if it is disabled via
     * {@link CanandgyroSettings#setYawFramePeriod(double)} it will not return fresh data.
     * 
     * @return multi-turn yaw in rotations.
     */
    public double getMultiturnYaw() {
        return multiYaw.getData();
    }

    /**
     * Gets the pitch (Y-axis) rotation from [-0.5 inclusive..0.5 exclusive).
     * 
     * @return pitch in rotations.
     */
    public double getPitch() {
        return quat.getPitch();
    }

    /**
     * Gets the roll (Z-axis) rotation from [-0.5 inclusive..0.5 exclusive).
     * 
     * @return roll in rotations.
     */
    public double getRoll() {
        return quat.getRoll();
    }

    /**
     * Gets the angular velocity along the roll (X) axis in rotations per second.
     * @return angular velocity in rot/s
     */
    public double getAngularVelocityRoll() {
        return vel.getX();
    }

    /**
     * Gets the angular velocity along the pitch (Y) axis in rotations per second.
     * @return angular velocity in rot/s
     */
    public double getAngularVelocityPitch() {
        return vel.getY();
    }

    /**
     * Gets the angular velocity along the yaw (Z) axis in rotations per second.
     * @return angular velocity in rot/s
     */
    public double getAngularVelocityYaw() {
        return vel.getZ();
    }

    /**
     * Gets the linear acceleration along the X axis in gravitational units.
     * @return linear acceleration in Gs
     */
    public double getAccelerationX() {
        return accel.getX();
    }

    /**
     * Gets the linear acceleration along the Y axis in gravitational units.
     * @return linear acceleration in Gs
     */
    public double getAccelerationY() {
        return accel.getY();
    }

    /**
     * Gets the linear acceleration along the Z axis in gravitational units.
     * @return linear acceleration in Gs
     */
    public double getAccelerationZ() {
        return accel.getZ();
    }

    /**
     * Gets the dedicated single-turn yaw {@link Frame} object.
     * @return yaw frame
     */
    public DoubleFrame<Double> getYawFrame() {
        return singleYaw;
    }

    /**
     * Gets the dedicated multi-turn yaw {@link Frame} object.
     * @return yaw frame
     */
    public DoubleFrame<Double> getMultiturnYawFrame() {
        return multiYaw;
    }

    /**
     * Gets the angular position {@link Frame} object.
     * @return angular position quaternion frame
     */
    public QuaternionFrame getAngularPositionFrame() {
        return quat;
    }

    /**
     * Gets the angular velocity {@link Frame} object.
     * 
     * getValue() returns a <code>Vec&lt;N3&gt;</code> in roll/pitch/yaw order in rotations per second.
     * @return angular velocity frame
     */
    public Vec3Frame getAngularVelocityFrame() {
        return vel;
    }

    /**
     * Gets the linear acceleration {@link Frame} object.
     * 
     * getValue() returns a <code>Vec&lt;N3&gt;</code> in x/y/z order in gravitational units.
     * @return acceleration frame
     */
    public Vec3Frame getAccelerationFrame() {
        return accel;
    }
    /**
     * Returns the current status frame which includes CAN timestamp data.
     * @return the current status frame as a {@link CanandgyroStatus} record.
     */
    public Frame<CanandgyroStatus> getStatusFrame() {
        return status;
    }


    private int quat2U16(double v) {
        int ret = (int) (v * 32767);
        if (ret > 32767) ret = 32767;
        if (ret < -32767) ret = -32767;
        return ret;
    }

    /**
     * Tell the Canandgyro to begin its calibration routine.
     * 
     * This calibration routine is performed automatically on power-on and takes several seconds.
     * The LED on the Canandgyro will stay at a solid yellow during the calibration process.
     * As this method returns immidiately, it is up to the user code to determine if 
     * the device is done calibrating (e.g. through {@link #isCalibrating()}).
     */
    public void startCalibration() {
        calibrating.set(true);
        sendCANMessage(CanandgyroDetails.Msg.kCalibrate, 0L, 8);
    }

    /**
     * Returns if the Canandgyro is known to be currently calibrating.
     * @return if the Canandgyro is calibrating
     */
    public boolean isCalibrating() {
        return calibrating.get();
    }

    /**
     * Blocks the current thread until the Canandgyro has finished calibrating or until a timeout is reached.
     * 
     * @param timeout the timeout in seconds to wait for a calibration confirmation.
     * @return true if the calibration has finished within the timeout, false if not.
     */
    public boolean waitForCalibrationToFinish(double timeout) {
        if (timeout <= 0) {
            return !calibrating.get();
        }

        synchronized(calibrating) {
            try {
                calibrating.wait((long) (timeout * 1000), 0);
            } catch (InterruptedException e) {}
            return !calibrating.get();
        }
    }

    /**
     * Sets a new angular position pose without recalibrating with a given roll/pitch/yaw.
     * If you just want to set yaw, use {@link #setYaw(double)}
     * 
     * @param newRoll new roll (x) pose in rotations
     * @param newPitch new pitch (y) pose in rotations
     * @param newYaw new yaw (z) pose in rotations
     * @param timeout the timeout in seconds to wait for a pose set confirmation.
     *                Set to 0 to not check (always return true.)
     * @param retries how many times to attempt to retry the operation.
     * @return true if a pose set confirmation was received (or if timeout is zero)
     */
    public boolean setPose(double newRoll, double newPitch, double newYaw, double timeout, int retries) {
        return setPose(new Rotation3d(newRoll * TAU, newPitch * TAU, newYaw * TAU).getQuaternion(), timeout, retries);
    }

    /**
     * Sets a new angular position without recalibrating with a {@link Rotation3d}.
     * 
     * @param newPose new rotation3d pose
     * @param timeout the timeout in seconds to wait for a pose set confirmation.
     *                Set to 0 to not check (always return true.)
     * @return true if a pose set confirmation was received (or if timeout is zero)
     */
    public boolean setPose(Rotation3d newPose, double timeout) {
        return setPose(newPose.getQuaternion(), timeout, 5);
    }

    /**
     * Sets a new pose without recalibrating with a {@link Quaternion}.
     * 
     * @param newPose new quaternion pose
     * @param timeout the timeout in seconds to wait for a pose set confirmation.
     *                Set to 0 to not check (always return true.)
     * @param retries how many times to attempt to retry the operation.
     * @return true if a pose set confirmation was received (or if timeout is zero)
     */
    public boolean setPose(Quaternion newPose, double timeout, int retries) {
        newPose = newPose.normalize();
        int idxToSet = (newPose.getW() >= 0) ? CanandgyroDetails.Stg.kSetPosePositiveW : CanandgyroDetails.Stg.kSetPoseNegativeW;

        boolean success = false;
        for (int i = 0; i < retries && !success; i++) {
            success = stg.confirmSetSetting(idxToSet, 
            // it's the same format irrespective of positive or negative W
            // since quaternions here are assumed to be norm 1, we have two packets to let us transmit
            // a quaternion in 6 bytes and work out what W is from re-norming on the other end
            CanandgyroDetails.Stg.constructSetPosePositiveW(
                quat2U16(newPose.getX()),
                quat2U16(newPose.getY()),
                quat2U16(newPose.getZ())
            ), timeout, 0).isValid();
        }
        return success;
    }

    /**
     * Sets a new yaw without recalibrating the Canandgyro.
     * Blocks for up to 100 milliseconds across 5 tries to confirm the transaction.
     * @param yaw new yaw angle in rotations
     * @return true if a confirmation was received
     */
    public boolean setYaw(double yaw) {
        return setYaw(yaw, 0.1, 5);
    }

    /**
     * Sets a new yaw without recalibrating the Canandgyro.
     * 
     * @param yaw new yaw angle in rotations
     * @param timeout the timeout in seconds to block to confirm the transaction (set 0 to not block)
     * @param retries how many times to attempt to retry the operation.
     * @return true if a confirmation was received or the timeout is zero
     */
    public boolean setYaw(double yaw, double timeout, int retries) {
        // wraparounds counts how many times we've rotated past the plus/minus 180 degree point.
        // so to convert whole rotations into this format, we need to add/subtract 0.5 rotations so 
        // that fractional portions roll over properly at the boundary.
        double offset = Math.copySign(0.5, yaw);
        yaw += offset;
        int wraparounds = (int) yaw;
        yaw = yaw - (double) wraparounds - offset;

        boolean success = false;
        for (int i = 0; i < retries && !success; i++) {
            success = stg.confirmSetSetting(
                CanandgyroDetails.Stg.kSetYaw, 
                CanandgyroDetails.Stg.constructSetYaw((float) (yaw * TAU), wraparounds),
                timeout, 0).isValid();
        }
        return success;
    }

    /**
     * Returns sticky faults.
     * Sticky faults are the active faults, except once set they do not become unset until 
     * {@link #clearStickyFaults()} is called.
     * 
     * @return {@link CanandgyroFaults} of the sticky faults.
     * @see #getActiveFaults()
     */
    public CanandgyroFaults getStickyFaults() {
        return status.getValue().stickyFaults();
    }

    /**
     * Returns an object representing currently active faults.
     * Active faults are only active for as long as the error state exists.
     * 
     * @return {@link CanandgyroFaults} of the active faults
     * @see #getStickyFaults()
     */
    public CanandgyroFaults getActiveFaults() {
        return status.getValue().activeFaults();
    }
    /**
     * Get onboard device temperature readings in degrees Celsius.
     * @return temperature in degrees Celsius
     */
    public double getTemperature() {
        return status.getValue().temperature();
    }

    /**
     * Get the contents of the previous status packet, which includes active faults, sticky faults, and temperature.
     * @return device status as a {@link CanandgyroStatus} record
     */
    public CanandgyroStatus getStatus() {
        return status.getValue();
    }

    /**
     * Clears sticky faults. 
     * 
     * <p>It is recommended to clear this during initialization, so one can check if the device
     * loses power during operation later. </p>
     * <p>This call does not block, so it may take up to the next status frame (default every 1000 
     * ms) for the sticky faults to be updated. To check for validity, use 
     * {@link CanandgyroFaults#faultsValid()} for faults returned by {@link #getStickyFaults()}</p>
     */
    public void clearStickyFaults() {
        synchronized(status) {
            sendCANMessage(CanandgyroDetails.Msg.kClearStickyFaults, 0, 1);
            // reset status framedata such that faults are now invalid again
            status.clearData();
        }
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
        sendCANMessage(CanandgyroDetails.Msg.kPartyMode, level, 1);
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
     * Canandgyro gyro = new Canandgyro(0); 
     * 
     * // Typical usage
     * // fetch all settings with a timeout of 320 ms, and retry missing values 3 times
     * CanandgyroSettings stg = gyro.getSettings(0.350, 0.1, 3);
     * 
     * // Advanced usage
     * gyro.startFetchSettings(); // send a "fetch settings command"
     * 
     * // wait some amount of time
     * stg = gyro.getSettingsAsync();
     * stg.allSettingsReceived(); // may or may not be true
     * 
     * stg = gyro.getSettings(0, 0.1, 3); // only fetch the missing settings
     * stg.allSettingsReceived(); // far more likely to be true
     * </pre>
     * 
     * <p> <b>Note that this function may return incomplete settings!</b> 
     * Use {@link CanandgyroSettings#allSettingsReceived()} to verify all settings were received.
     * 
     * @param timeout maximum number of seconds to wait for settings before giving up.
     * @param missingTimeout maximum number of seconds to wait for each settings retry before giving up.
     * @param attempts number of attempts to try and fetch values missing from the first pass
     * @return {@link CanandgyroSettings} representing the device's configuration
     */
    public CanandgyroSettings getSettings(double timeout, double missingTimeout, int attempts) {
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
     * Use {@link CanandgyroSettings#allSettingsReceived()} to verify all settings were received.
     * 
     * @param timeout maximum number of seconds to wait for settings before giving up
     * @return {@link CanandgyroSettings} representing the device's configuration
     */
    public CanandgyroSettings getSettings(double timeout) {
        return stg.getSettings(timeout, 0.1, 1);
    }

    /**
     * Fetches the device's current configuration in a blocking manner.
     * This function will block for up to 0.350 seconds waiting for the device to reply, so it is 
     * best to put this in an init function rather than the main loop.
     * 
     * <p> <b>Note that this function may return incomplete settings!</b> 
     * Use {@link CanandgyroSettings#allSettingsReceived()} to verify all settings were received.
     * @return {@link CanandgyroSettings} representing the device's configuration
     */
    public CanandgyroSettings getSettings() {
        return getSettings(0.350);
    }

        /**
     * Tells the Canandgyro to begin transmitting its settings; once they are all transmitted 
     * (after ~200-300ms), the values can be retrieved from {@link Canandgyro#getSettingsAsync()}
     */
    public void startFetchSettings() {
        stg.startFetchSettings();
    }

    /**
     * Non-blockingly returns a {@link CanandgyroSettings} object of the most recent known 
     * settings values received from the device.
     *
     * <p><b>Most users will probably want to use {@link Canandgyro#getSettings()} instead. </b></p>
     * 
     * One can call this after a {@link Canandgyro#startFetchSettings()} call, and use 
     * {@link CanandgyroSettings#allSettingsReceived()} to check if/when all values have been 
     * seen. As an example:
     * 
     * <pre>
     * 
     * // somewhere in an init function
     * Canandgyro canandgyro = new Canandgyro(0); 
     * canandgyro.startFetchSettings();
     * 
     * // ...
     * // somewhere in a loop function
     * 
     * if (canandgyro.getSettingsAsync().allSettingsReceived()) {
     *   // do something with the settings object
     *   System.out.printf("Canandgyro yaw frame period: %d\n",
     *      canandgyro.getSettingsAsync().getYawFramePeriod());
     * }
     * </pre>
     * 
     * 
     * If this is called after {@link Canandgyro#setSettings(Canandgyro.CanandgyroSettings)}, this method 
     * will return a settings object where only the fields where the device has echoed the new 
     * values back will be populated. To illustrate this, consider the following:
     * <pre>
     * 
     * // somewhere in initialization (just as a definition):
     * Canandgyro canandgyro = new Canandgyro(0); 
     * 
     * // somewhere in a loop 
     * canandgyro.setSettings(new CanandgyroSettings().setStatusFramePeriod(0.100));
     * 
     * // will likely return Empty, as the device hasn't confirmed the previous transaction
     * canandgyro.getSettingsAsync().getStatusFramePeriod(); 
     * 
     * // after up to ~300 ms...
     * canandgyro.getSettingsAsync().getStatusFramePeriod(); // will likely return 100 ms
     * </pre>
     * 
     * @see Canandgyro#startFetchSettings
     * @return CanandgyroSettings object of known settings
     */
    public CanandgyroSettings getSettingsAsync() {
        return stg.getKnownSettings();
    }

    /**
     * Applies the settings from a {@link CanandgyroSettings} object to the device, with fine
     * grained control over failure-handling.
     * 
     * This overload allows specifiyng the number of retries per setting as well as the confirmation
     * timeout. Additionally, it returns a {@link CanandgyroSettings} object of settings that 
     * were not able to be successfully applied.
     * 
     * @see CanandgyroSettings
     * @param settings the {@link CanandgyroSettings} to update the encoder with
     * @param timeout maximum time in seconds to wait for each setting to be confirmed. Set to 0 to 
     *     not check (and not block).
     * @param attempts the maximum number of attempts to write each individual setting
     * @return a CanandgyroSettings object of unsuccessfully set settings.
     */
    public CanandgyroSettings setSettings(CanandgyroSettings settings, double timeout, int attempts) {
        return stg.setSettings(settings, timeout, attempts);
    }

    /**
     * Applies the settings from a {@link CanandgyroSettings} object to the device. 
     * For more information, see the {@link CanandgyroSettings} class documentation.
     * @see CanandgyroSettings
     * @param settings the {@link CanandgyroSettings} to update the device with
     * @param timeout maximum time in seconds to wait for each setting to be confirmed. Set to 0 to
     *     not check (and not block).
     * @return true if successful, false if a setting operation timed out
     */
    public boolean setSettings(CanandgyroSettings settings, double timeout) {
        return stg.setSettings(settings, timeout);
    }

    /**
     * Applies the settings from a {@link CanandgyroSettings} object to the device. 
     * For more information, see the {@link CanandgyroSettings} class documentation.
     * @see CanandgyroSettings
     * @param settings the {@link CanandgyroSettings} to update the device with
     * @return true if successful, false if a setting operation timed out
     */
    public boolean setSettings(CanandgyroSettings settings) {
        return setSettings(settings, 0.10);
    }

    /**
     * Resets the device to factory defaults. 
     * @param timeout how long to wait for the new settings to be confirmed by the device in seconds
     *     (suggested at least 0.35 seconds)
     * @return {@link CanandgyroSettings} object of received settings. 
     *     Use {@link CanandgyroSettings#allSettingsReceived()} to verify success.
     */
    public CanandgyroSettings resetFactoryDefaults(double timeout) {
        return stg.sendReceiveSettingCommand(CanandgyroDetails.Enums.SettingCommand.kResetFactoryDefault, timeout, true);
    }

    /**
     * Returns the {@link CanandSettingsManager} associated with this device.
     * 
     * The {@link CanandSettingsManager} is an internal helper object. 
     * Teams are typically not expected to use it except for advanced cases (e.g. custom settings
     * wrappers)
     * @return internal settings manager handle
     */
    public CanandSettingsManager<CanandgyroSettings> getInternalSettingsManager() {
        return stg;
    }

    @Override
    public void handleMessage(CanandMessage msg) {
        byte[] data = msg.getData();
        double timestamp = msg.getTimestamp();
        switch(msg.getApiIndex()) {
            case CanandgyroDetails.Msg.kYawOutput: {
                if (msg.getLength() != CanandgyroDetails.Msg.kDlc_YawOutput) break;
                long dataAsLong = msg.getDataAsLong();
                //yaw.updateData(CanandgyroDetails.Msg.extractYawOutput_Yaw(0))
                double angle = CanandgyroDetails.Msg.extractYawOutput_Yaw_Yaw(dataAsLong);
                int wrap = CanandgyroDetails.Msg.extractYawOutput_Yaw_Wraparound(dataAsLong);
                multiYaw.updateData(angle / TAU + (double) wrap, timestamp);
                singleYaw.updateData(angle / TAU, timestamp);
                break;
            }
            case CanandgyroDetails.Msg.kAngularPositionOutput: {
                if (msg.getLength() != CanandgyroDetails.Msg.kDlc_AngularPositionOutput) break;
                quat.updateData(data, timestamp);
                break;
            }
            case CanandgyroDetails.Msg.kAngularVelocityOutput: {
                if (msg.getLength() != CanandgyroDetails.Msg.kDlc_AngularVelocityOutput) break;
                vel.updateData(data, timestamp);
                break;
            }
            case CanandgyroDetails.Msg.kAccelerationOutput: {
                if (msg.getLength() != CanandgyroDetails.Msg.kDlc_AccelerationOutput) break;
                accel.updateData(data, timestamp);
                break;
            }
            case CanandgyroDetails.Msg.kCalibrationStatus: {
                synchronized(calibrating) {
                    calibrating.set(false);
                    calibrating.notifyAll();
                }
                break;
            }
            case CanandgyroDetails.Msg.kStatus: {
                if (msg.getLength() != CanandgyroDetails.Msg.kDlc_Status) break;
                status.updateData(data, timestamp);
                if (calibrating.get() && ((int) data[0] & CanandgyroDetails.Bitsets.kFaults_Calibrating) == 0) {
                    synchronized(calibrating) {
                        calibrating.set(false);
                        calibrating.notifyAll();
                    }
                }

                break;
            }
            case CanandgyroDetails.Msg.kReportSetting: {
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
    } // end canandgyro
    
}
