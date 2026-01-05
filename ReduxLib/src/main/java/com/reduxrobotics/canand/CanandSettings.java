// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.canand;

import java.util.Map;
import java.util.Optional;
import java.util.HashMap;
import java.util.ArrayList;
import java.util.List;

/**
 * Base (simple) settings class for Redux devices.
 * 
 * Inheriting classes with more complex firmware interfaces may or may not use this structure.
 * It's typically used in conjunction with CanandSettingsManager.
 * 
 * In general, however, it's a pretty useful structure.
 */
public abstract class CanandSettings {

    /**
     * Default constructor.
     */
    public CanandSettings() {}

    /**
     * Clone constructor.
     * @param stg the other object to clone.
     */
    public CanandSettings(CanandSettings stg) {
        values = stg.getFilteredMap();
    }
    
    /**
     * The backing Map of raw setting indexes to raw setting values (serialized as Long)
     */
    protected Map<Integer, Long> values = new HashMap<>();

    /**
     * Gets the list of settings addresses this settings class records.
     * This is typically a static list of constants from a Details class.
     * 
     * @return settings addresses.
     */
    protected abstract int[] fetchSettingsAddresses();

    /**
     * Whether the settings will be applied ephemerally.
     */
    protected boolean ephemeral = false;

    /**
     * Check bounds for an input double that is mapped to some integer type underneath.
     * 
     * A common example is to map some value between [0..1) to [0..16383].
     * 
     * @param flavor Flavor text describing the value
     * @param rawVal The raw inputted double value
     * @param min The minimum valid integer value
     * @param max The maximum value integer value
     * @param scale How much the raw value must be multiplied by to fit the scale
     * @return A scaled integer (long) value mapped from the input raw value.
     */
    protected static long checkBounds(String flavor, double rawVal, int min, int max, double scale) {
        long val = (long) (rawVal * scale);
        if (val < min || val > max) {
            throw new IllegalArgumentException(
                String.format("value %.3f is not in the valid %s range [%.3f..%.3f]", 
                rawVal, flavor, min / scale, max / scale));
        }
        return val;
    }

    /**
     * Gets an integer from the internal Map if it exists and return a scaled user value.
     * 
     * @param idx the setting index to pull
     * @param divisor how much to divide the integer value by
     * @return a scaled double if the setting exists else null
     */
    protected Optional<Double> getIntAsDouble(int idx, double divisor) {
        if (!values.containsKey(idx)) return Optional.empty();
        return Optional.of(values.get(idx).intValue() / divisor);
    }

    /**
     * Gets an integer from the internal Map if it exists and return a scaled user value.
     * 
     * @param idx the setting index to pull
     * @param divisor how much to divide the float value by
     * @return a scaled double if the setting exists else null
     */
    protected Optional<Double> getFloatAsDouble(int idx, double divisor) {
        if (!values.containsKey(idx)) return Optional.empty();
        return Optional.of(Float.intBitsToFloat(values.get(idx).intValue()) / divisor);
    }
    
    /**
     * Gets a boolean from the internal Map if it exists and return true, false, or null
     * 
     * @param idx the setting index to pull
     * @return a boolean if the setting exists else null
     */
    protected Optional<Boolean> getBool(int idx) {
        if (!values.containsKey(idx)) return null;
        return Optional.of(values.get(idx) == 1L);
    }


    /**
     * Returns whether or not all settings fields have been written into the object. 
     * 
     * <p>May return false if the a getSettings call did not succeed in fetching every setting.
     * 
     * @return whether the settings object has been filled
     */
    public boolean allSettingsReceived() {
        for (int i : fetchSettingsAddresses()) {
            if (!values.containsKey(i)) {
                return false;
            }
        }
        return true;
    }

    /**
     * Directly access the underlying {@link Map} of settings values. Intended for internal use.
     * @return map
     */
    public Map<Integer, Long> getMap() {
        return values;
    }

    /**
     * Return a direct filtered view of settings values as a new {@link Map}, limited to only valid
     * settings.
     * @return map
     */
    public Map<Integer, Long> getFilteredMap() {
        HashMap<Integer, Long> ret = new HashMap<>();
        for (int i : fetchSettingsAddresses()) {
            if (values.containsKey(i)) {
                ret.put(i, values.get(i));
            }
        }
        return ret;
    }

    /**
     * Returns if this CanandSettings has any set settings or not.
     * Useful when a CanandSettings is returned as a result of setSettings to check if all settings 
     * succeeded.
     * @return true if empty
     */
    public boolean isEmpty() {
        return values.isEmpty();
    }

    /**
     * Sets whether or not the settings will be set as ephemeral -- that is, does not persist on
     * device power cycle.
     * 
     * <p>
     * This is useful if you wish for the settings to:
     * </p>
     * <ul>
     * <li>be updated frequently e.g. to work around device API limitations without causing flash wear</li>
     * <li>be used to signal things even across robot program restarts (e.g. with scratch settings)</li>
     * </ul>
     * 
     * <p>
     * Generally, Redux devices will skip writing to flash if the sent setting already equals what
     * is stored in flash, so in practice, if you are only applying a fixed set of settings on robot startup,
     * there is little benefit to sending them ephemerally as it does not affect flash wear if the
     * settings already match, and sending them ephemerally reduces robustness against things like 
     * intermittent power connectivity and power cycling of the device. 
     * </p>
     * 
     * @param value true if ephemeral
     */
    public void setEphemeral(boolean value) {
        ephemeral = value;
    }

    /**
     * Returns true if these settings are to be set ephemerally.
     * @return true if ephemeral
     */
    public boolean isEphemeral() {
        return ephemeral;
    }

    /**
     * Generates a list of setting indexes missing from this object to be considered "complete".
     * @return List of missing setting indexes.
     */
    public List<Integer> getMissingIndexes() {
        List<Integer> missing = new ArrayList<>();
        for (int addr: fetchSettingsAddresses()) {
            if (!values.containsKey(addr)) {
                missing.add(addr);
            }
        }
        return missing;
    }

    @Override
    public String toString() {
        StringBuilder builder = new StringBuilder();
        builder.append(getClass().getName());
        builder.append(" {\n");
        for (int i: fetchSettingsAddresses()) {
            int ii = i & 0xff;
            if (values.containsKey(i)) {
                builder.append(String.format("  0x%x | %d : 0x%x,\n", ii, ii, values.get(i)));
            } else {
                builder.append(String.format("  0x%x | %d : null,\n", ii, ii));
            }
        }
        builder.append("}");
        return builder.toString();
    }
}
