// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.canand;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;

import edu.wpi.first.wpilibj.DriverStation;

/**
 * Common logic for settings management for {@link CanandDevice}s.
 * 
 * This class holds a {@link CanandSettings} cache of known settings received from the CAN bus,
 * and offers a series of helper functions that provide common logic for bulk settings operations.
 * @param <T>the specialized settings object type that this manages
 */
public class CanandSettingsManager<T extends CanandSettings> {
    /** Bit flag specifying this setting should be set ephemeral., */
    public static final int kFlag_Ephemeral = 1;

    private T knownSettings;
    private CanandSettingsCtor<T> ctor;
    private CanandDevice dev;

    /** Object used as a synchronization flag for individual setting receives. */
    private final Object settingRecvFlag = new Object();
    /** The Long value of the last received setting. */
    private long settingRecvValue = 0;

    /** The byte index of the last received setting. */
    private int settingRecvIdx = 0;
    /** The index flags of the last received setting. */
    private int settingRecvCode = 0;


    /**
     * Counter of how many settings have been received at runtime.
     * Used to check if new settings have actually been received or if reentrancy is spurious
     */
    private int settingRecvCtr = 0;

    private int[] settingsSubset;

    /**
     * Functional interface for a CanandSettings constructor.
     * 
     * This interface exists as a workaround for Java's type erasure.
     * @param <T>the specialized settings object type that this manages
     */
    public static interface CanandSettingsCtor<T extends CanandSettings> { 
        /**
         * Constructor function.
         * @return new CanandSettings subclass
         */
        T construct(); 
    }

    /**
     * Setting result codes.
     * 
     * Positive indicates codes returnable from the "report setting" packet, while negative 
     * indicates codes returnable from other causes (e.g. timeouts)
     */
    public static enum ResultCode {
        /** General invalid data (a placeholder) */
        kInvalid(-1),
        /** Operation timeout */
        kTimeout(-2),
        /** Success */
        kError(0),
        /** General error */
        kOk(1);
        private int index;
        private ResultCode(int index) {
            this.index =index;
        }
        /**
         * Return associated index (unsigned for values >= 0).
         * @return index value
         */
        public int getIndex() {
            return index;
        }
        //private static ResultCode positive[] = {kOk, kError};
        private static ResultCode negative[] = {kInvalid, kTimeout};

        /**
         * Return the appropirate enum from the index value.
         * @param index index value
         * @return corresponding enum.
         */
        public static ResultCode fromIndex(int index) {
            // This technically doesn't support the whole spec but this is Good Enough for vdep purposes
            if ((index & 0b1) > 0) {
                return kOk;
            } else if (index == 0) {
                return kError;
            } else {
                return negative[(-index + 1) % negative.length];
            }
        }
    }

    /**
     * Record of setting results.
     * 
     * @param value the setting value, as a long
     * @param result the result code associated with the operation.
     */
    public static record SettingResult(long value, ResultCode result) {
        /**
         * Copies the long-backed value to a byte array.
         * 
         * @param out byte array
         */
        public void toByteArray(byte[] out) {
            for (int i = 0; i < Math.min(6, out.length); i++) {
                out[i] = (byte) ((value >> (i << 8)) & 0xff);
            }
        }

        /**
         * Returns true if the result is valid.
         * 
         * @return true on valid, false on invalid.
         */
        public boolean isValid() {
            return result == ResultCode.kOk;
        }
        /** Invalid singleton. */
        public static final SettingResult INVALID = new SettingResult(-1, ResultCode.kInvalid);
        /** Timeout singleton. */
        public static final SettingResult TIMEOUT = new SettingResult(-1, ResultCode.kTimeout);
    }

    /**
     * Construct a new CanandSettingsManager.
     * @param dev the device to be associated with this object.
     * @param ctor the constructor of the CanandSettings subclass. 
     */
    public CanandSettingsManager(CanandDevice dev, CanandSettingsCtor<T> ctor) {
        this.dev = dev;
        this.ctor = ctor;
        this.knownSettings = ctor.construct();
        this.settingsSubset = this.knownSettings.fetchSettingsAddresses();
    }

    /**
     * Fetches a setting subset given the setting command to request with and
     * list of indexes that are expected to be received.
     * @param settingCmdIndex the setting command index to send
     * @param subset the list of setting indexes to expect to receive
     * @param timeout in seconds for settings to be received
     * @param missingTimeout maximum number of seconds to wait for each settings retry before giving up
     * @param missingAttempts if >0, lists how many times to attempt fetching missing settings.
     * @return {@link CanandSettings} representing what has been received of the device's 
     *     configuration.
     */
    public T getSettingSubset(int settingCmdIndex, int[] subset, double timeout, double missingTimeout, int missingAttempts) {
        synchronized(this) {
            boolean success = false;
            if (timeout > 0) {
                for (int i : subset) {
                    knownSettings.getMap().remove(i);
                }
                settingsSubset = subset;
                sendSettingCommand(settingCmdIndex);
                success = waitSubsetReceived(subset, missingTimeout);
            }
            if (success || missingAttempts < 1 || missingTimeout <= 0) return getKnownSettings();
        }
        fetchMissingSettingsSubset(subset, missingTimeout, missingAttempts);
        return getKnownSettings();
    }

    private boolean subsetReceived(int[] subset) {
        for (int i : subset) {
            if (!knownSettings.getMap().containsKey(i)) return false;
        }
        return true;
    }

    private synchronized boolean waitSubsetReceived(int[] subset, double timeout) {
        try {
            double before = CanandUtils.getFPGATimestamp();
            //System.out.println("before: " + before);
            while (!subsetReceived(subset)) {
                if (CanandUtils.getFPGATimestamp() - before > timeout) return false;
                this.wait((long) (timeout * 1000));
            }
        } catch (InterruptedException e) { return subsetReceived(subset); }
        return true;
    }

    /**
     * Fetches a list of settings that are missing from the known settings set.
     * 
     * @param subset list of settings that the known settings cache is to be filled with.
     * @param timeout how long to wait to fetch individual settings that were not received
     * @param attempts how many times to attempt fetching each setting that has not been received.
     * @return a list of setting indexes that are still missing, despite attempts
     */
    public List<Integer> fetchMissingSettingsSubset(int[] subset, double timeout, int attempts) {
        List<Integer> missingNow = new ArrayList<>();
        List<Integer> missingFinal = new ArrayList<>();

        synchronized (this) {
            // We get a synchronized snapshot of missing keys.
            for (int addr : subset) {
                if (!knownSettings.getMap().containsKey(addr)) {
                    missingNow.add(addr);
                }
            }
        }

        // we don't want to synchronize because the device handler should call handleSetting
        // and we'll ingest the setting there.
        // otherwise we'll just hang the CanandEventLoop
        for (Integer addr: missingNow) {
            SettingResult value = SettingResult.INVALID;
            for (int i = 0; i < attempts && !value.isValid(); i++) {
                value = fetchSetting(addr, timeout);
            }
            if (!value.isValid()) missingFinal.add(addr);
        }
        return missingFinal;
    }

    /**
     * Fetches the device's current configuration in a blocking manner.
     * 
     * This function will block for at least `timeout` seconds waiting for the device to reply, so 
     * it is best to put this in a teleop or autonomous init function, rather than the main loop.
     * 
     * @param timeout maximum number of seconds to wait for the all settings pass before giving up
     * @param missingTimeout maximum number of seconds to wait for each settings retry before giving up
     * @param missingAttempts if >0, lists how many times to attempt fetching missing settings.
     * @return {@link CanandSettings} representing what has been received of the device's 
     *     configuration.
     */
    public T getSettings(double timeout, double missingTimeout, int missingAttempts) {
        return getSettingSubset(
            CanandDeviceDetails.Enums.SettingCommand.kFetchSettings,
            knownSettings.fetchSettingsAddresses(), 
            timeout,
            missingTimeout,
            missingAttempts
        );
    }

    /**
     * Tells the device to begin transmitting its settings. 
     * Once they are all transmitted (typically after ~200-300ms),
     * the values can be retrieved from {@link #getKnownSettings()}
     */
    public synchronized void startFetchSettings() {
        // send a settings request
        sendSettingCommand(CanandDeviceDetails.Enums.SettingCommand.kFetchSettings);
        knownSettings.getMap().clear();
    }

    /**
     * Applies the settings from a {@link CanandSettings} object to the device, with fine
     * grained control over failure-handling.
     * 
     * This overload allows specifiyng the number of retries per setting as well as the confirmation
     * timeout. Additionally, it returns a {@link CanandSettings} object of settings that 
     * were not able to be successfully applied.
     * 
     * @param settings the {@link CanandSettings} to update the device with
     * @param timeout maximum time in seconds to wait for each setting to be confirmed. Set to 0 to 
     *     not check (and not block).
     * @param attempts the maximum number of attempts to write each individual setting
     * @return a CanandSettings object of unsuccessfully set settings.
     */
    public T setSettings(T settings, double timeout, int attempts) {
        T missed_settings = ctor.construct();
        Map<Integer, Long> values = settings.getFilteredMap();
        int flags = 0;
        if (settings.ephemeral) {
            flags |= kFlag_Ephemeral;
        }
        for (int addr : values.keySet()) {
            synchronized (this) {
                knownSettings.getMap().remove(addr);
            }
            boolean success = false;
            for (int i = 0; i < attempts && !success; i++) {
                success = confirmSetSetting(addr, values.get(addr), timeout, flags).isValid();
            }
            if (!success) {
                // Add the missed setting to the missed settings map
                missed_settings.getMap().put(addr, values.get(addr));
            }
        }

        return missed_settings;
    }

    /**
     * Applies the settings from a {@link CanandSettings} object to the device. 
     * 
     * @param settings the {@link CanandSettings} to update the device  with
     * @param timeout maximum time in seconds to wait for each setting to be confirmed. Set to 0 to 
     *     not check (and not block).
     * @return true if successful, false if a setting operation failed
     */
    public boolean setSettings(T settings, double timeout) {
        T missed = setSettings(settings, timeout, 3);
        if (!missed.isEmpty()) {
            DriverStation.reportError(String.format("%d settings could not be applied to %s", 
                missed.getMap().size(), dev.toString()), true);
            return false;
        }
        return true;
    }

    /**
     * Runs a setting command that may mutate all settings and trigger a response.
     * 
     * Typically used with "reset factory default" type commands
     * @param cmd setting index
     * @param timeout total timeout for all settings to be returned
     * @param clearKnown whether to clear the set of known settings
     * @return the set of known settings.
     */
    public synchronized T sendReceiveSettingCommand(int cmd, double timeout, boolean clearKnown) {
        if (clearKnown) knownSettings.getMap().clear();
        settingsSubset = knownSettings.fetchSettingsAddresses();
        sendSettingCommand(cmd);
        waitAllSettings(timeout);
        return getKnownSettings();
    }

    /**
     * Return a CanandSettings of known settings.
     * The object returned is a copy of this object's internal copy.
     * @return known settings
     */
    public synchronized T getKnownSettings() {
        // construct a blank object and switch out backing map for a clone of knownSettings
        T ret = ctor.construct();
        ret.values = knownSettings.getFilteredMap();
        return ret;
    }

    /**
     * Blocks to wait for all settings to get sent. It is assumed that this function is called in 
     * a synchronized(this) block.
     * @param timeout timeout to wait in seconds
     * @return true if all settings suceeded
     */
    private boolean waitAllSettings(double timeout) {
        try {
            double before = CanandUtils.getFPGATimestamp();
            while (!knownSettings.allSettingsReceived()) {
                // timeout check
                if (CanandUtils.getFPGATimestamp() - before > timeout) return false;
                this.wait((long) (timeout * 1000));
            }
        } catch (InterruptedException e) { return false; }
        return true;
    }

    /**
     * Setting handler to put in {@link CanandDevice#handleMessage}.
     * 
     * Example:
     * <pre>
     * // boilerplate definition (in practice it won't be null)
     * CanandSettingsManager settingsMgr = null;
     * // from your handler:
     * CanandMessage msg = new CanandMessage();
     * 
     * // in the handler:
     * switch(msg.getApiIndex()) {
     *    case CanandDeviceDetails.Msg.kReportSetting: {
     *        if (settingsMgr == null) break;
     *        settingsMgr.handleSetting(msg);
     *        break;
     *    } 
     *    default: break;
     * }
     * </pre>
     * 
     * @param msg the CanandMessage containing the settings data to process (from handleMessage)
     */
    public void handleSetting(CanandMessage msg) {
        int length = msg.getLength();
        if (length < 7) return;
        byte[] data = msg.getData();
        int idx = ((int) data[0]) & 0xff;
        long settingValue = CanandUtils.extractLong(data, 8, 56, false); 
        synchronized (this) {
            knownSettings.getMap().put(idx, settingValue);
            if (subsetReceived(settingsSubset)) {
                this.notifyAll();
            }
        }
        synchronized (settingRecvFlag) {
            settingRecvValue = settingValue;
            settingRecvIdx = idx;
            settingRecvCtr++;
            if (length == 8) {
                settingRecvCode = data[7] & 0xff;
            } else {
                settingRecvCode = 0;
            }
            settingRecvFlag.notifyAll();
        }
    }

    /**
     * Directly sends a CAN message to the associated {@link CanandDevice} to set a setting by index.
     * This function does not block nor check if a report settings message is sent in response.
     * 
     * <p>
     * Device subclasses will usually have a more user-friendly settings interface, 
     * eliminating the need to call this function directly in the vast majority of cases.
     * </p>
     * 
     * @param settingId the setting id
     * @param value the raw numerical value. Only the lower 48 bits will be used.
     * @param flags optional flags to send to the device specifying how the setting will be set.
     */
    public void setSettingById(int settingId, long value, int flags) {
        dev.sendCANMessage(CanandDeviceDetails.Msg.kSetSetting, 
            (settingId & 0xff) | 
            ((value & 0xffffffffffffL) << 8) | 
            ((flags & 0xff) << 56),
        8);
    }

    /**
     * Directly sends a CAN message to the associated {@link CanandDevice} to set a setting by index. 
     * This function does not block nor check if a report settings message is sent in response.
     * 
     * <p>
     * Device subclasses will usually have a more user-friendly settings interface, 
     * eliminating the need to call this function directly in the vast majority of cases.
     * </p>
     * 
     * @param settingId the setting id
     * @param value a value byte array. Up to the first 6 bytes will be used.
     * @param flags optional flags to send to the device specifying how the setting will be set.
     */
    public void setSettingById(int settingId, byte[] value, int flags) {
        // perform actual send logic here
        byte data[] = {(byte) settingId, 0, 0, 0, 0, 0, 0};
        for (int i = 0; i < Math.min(6, value.length); i++) {
            data[i + 1] = value[i];
        }
        dev.sendCANMessage(CanandDeviceDetails.Msg.kSetSetting, data);
    }

    /**
     * Potentially blocking operation to send a setting and wait for a report setting message to be 
     * received to confirm the operation, with retries.
     * 
     * @param settingIdx Setting index to set and listen for
     * @param payload the long value to send. Only lower 48 bits are used.
     * @param timeout the timeout to wait before giving up in seconds. Passing in 0 will return 
     *     instantly (not block)
     * @param flags optional flags to send to the device specifying how the setting will be set.
     * @param attempts number of times to attempt a retry
     * @return the value received by the report setting packet if existent or 
     *     {@link SettingResult#TIMEOUT} otherwise. If timeout = 0, return "payload" (assume success)
     */
    public SettingResult confirmSetSetting(int settingIdx, long payload, double timeout, int flags, int attempts) {
        SettingResult result = SettingResult.TIMEOUT;
        for (int i = 0; i < attempts; i++) {
            result = confirmSetSetting(settingIdx, payload, timeout, flags);
            if (result.isValid()) {
                return result;
            }
        }
        return result;
    }

    /**
     * Potentially blocking operation to send a setting and wait for a report setting message to be 
     * received to confirm the operation.
     * 
     * @param settingIdx Setting index to set and listen for
     * @param payload the long value to send. Only lower 48 bits are used.
     * @param timeout the timeout to wait before giving up in seconds. Passing in 0 will return 
     *     instantly (not block)
     * @param flags optional flags to send to the device specifying how the setting will be set.
     * @return the value received by the report setting packet if existent or 
     *     {@link SettingResult#TIMEOUT} otherwise. If timeout = 0, return "payload" (assume success)
     */
    public SettingResult confirmSetSetting(int settingIdx, long payload, double timeout, int flags) {
        double initial = CanandUtils.getFPGATimestamp();
        double time = initial;
        double deadline = time + timeout;
        int prevCtr;
        synchronized (settingRecvFlag) {
            setSettingById(settingIdx, payload, flags);
            if (timeout <= 0) return new SettingResult(payload, ResultCode.kOk);
            do {
                try {
                    prevCtr = settingRecvCtr;
                    settingRecvFlag.wait(Math.max((long) (timeout * 1000), 1));
                    time = CanandUtils.getFPGATimestamp();
                    if (time > deadline) {
                        return SettingResult.TIMEOUT;
                    }
                } catch (InterruptedException e) { return SettingResult.TIMEOUT; }
            } while (settingIdx != settingRecvIdx || prevCtr == settingRecvCtr);
            return new SettingResult(settingRecvValue, ResultCode.fromIndex(settingRecvCode));
        }
    }
    
    /**
     * Fetches a setting from the device and returns the received result.
     * @param settingIdx Setting index to fetch
     * @param timeout timeout to wait before giving up in seconds. Passing in 0 will return 
     *    {@link SettingResult#TIMEOUT}.
     * @return {@link SettingResult} representing the setting result.
     */
    public SettingResult fetchSetting(int settingIdx, double timeout) {
        double time = CanandUtils.getFPGATimestamp();
        double deadline = time + timeout;
        int prevCtr;
        if (timeout <= 0) return SettingResult.TIMEOUT;
        synchronized (settingRecvFlag) {
            dev.sendCANMessage(
                CanandDeviceDetails.Msg.kSettingCommand, 
                CanandDeviceDetails.Msg.constructSettingCommand(
                    CanandDeviceDetails.Enums.SettingCommand.kFetchSettingValue, 
                    settingIdx
                ),
                2
            );
            do {
                try {
                    prevCtr = settingRecvCtr;
                    settingRecvFlag.wait(Math.max((long) ((deadline - time) * 1000), 1));
                    time = CanandUtils.getFPGATimestamp();
                    if (time >= deadline) {
                        return SettingResult.TIMEOUT;
                    }
                } catch (InterruptedException e) { 
                    return SettingResult.TIMEOUT; 
                }
            } while (settingIdx != settingRecvIdx || prevCtr == settingRecvCtr);
            return new SettingResult(settingRecvValue, ResultCode.fromIndex(settingRecvCode));
        }
    }

    /**
     * Sends a setting command with no arguments.
     * @param settingCmdIdx the index of the setting command to send.
     */
    public void sendSettingCommand(int settingCmdIdx) {
        dev.sendCANMessage(CanandDeviceDetails.Msg.kSettingCommand, settingCmdIdx & 0xff, 1);
    }


}
