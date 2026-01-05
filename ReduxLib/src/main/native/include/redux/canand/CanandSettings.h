// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <cstdint>
#include <unordered_map>
#include <vector>
#include <fmt/format.h>

namespace redux::canand {

/**
 * Base (simple) settings class for Redux devices.
 * 
 * Inheriting classes with more complex firmware interfaces may or may not use this structure.
 * It's typically used in conjunction with CanandSettingsManager.
 * 
 * In general, however, it's a pretty useful structure.
 */
class CanandSettings {
  public:
    /** Default constructor. */
    CanandSettings() = default;
    /** 
     * Copy constructor -- only copies over a filtered copy of values. 
     * @param stg reference to another CanandSettings.
     */
    CanandSettings(CanandSettings& stg) : values{stg.FilteredMap()} {};

    /** Destructor. */
    ~CanandSettings() = default;

    /**
     * Return a direct filtered view of settings values as a new unordered_map, limited to only valid
     * settings.
     * @return map
     */
    inline std::unordered_map<uint8_t, uint64_t> FilteredMap() {
        std::unordered_map<uint8_t, uint64_t> ret;
        for (auto addr : SettingAddresses()) {
            if (values.contains(addr)) {
                ret[addr] = values[addr];
            }
        }
        return ret;
    }

    /**
     * Returns whether or not all settings fields have been written into the object. 
     * 
     * <p>May return false if the a getSettings call did not succeed in fetching every setting.
     * 
     * @return whether the settings object has been filled
     */
    inline bool AllSettingsReceived() const {
        for (auto addr : SettingAddresses()) {
            if (!values.contains(addr)) return false;
        } 
        return true;
    }

    /**
     * Gets the array of settings addresses this settings class records.
     * @return DetailsKey array
     */
    virtual const std::vector<uint8_t>& SettingAddresses() const {
        return details::VDEP_SETTINGS;
    };

    /**
     * Gets the backing store.
     * @return the underlying map.
     */
    inline std::unordered_map<uint8_t, uint64_t>& GetMap() {
        return values;
    }

    /**
     * Returns if this CanandSettings has any set settings or not.
     * Useful when a CanandSettings is returned as a result of setSettings to check if all settings 
     * succeeded.
     * @return true if empty
     */
    inline bool IsEmpty() const {
        return values.empty();
    }

    /**
     * Returns if this CanandSettings is set to be ephemeral.
     * 
     * Ephemeral settings do not persist on device reboot, but do not impose any flash wear.
     * @return true if ephemeral
     */
    inline bool IsEphemeral() const {
        return ephemeral;
    }

    /**
     * Sets whether or not the settings will be set as ephemeral -- that is, does not persist on
     * device power cycle.
     * 
     * Pre-v2024 firmwares will not support this!
     * 
     * @param value true if ephemeral
     */
    inline void SetEphemeral(bool value) {
        ephemeral = value;
    }

    /**
     * Dump the CanandSettings map as a string.
     * @return string
     */
    inline std::string ToString() {
        std::string s = "CanandSettings {\n";
        for (auto& it: values) {
            s.append(fmt::format("  0x{:x}: {:x},\n", it.first, it.second));
        }
        s.append("}");
        return s;
    }

  protected:
    /**
     * The backing store.
     */
    std::unordered_map<uint8_t, uint64_t> values;

    /**
     * Whether the settings in this class should be set ephemerally.
     */
    bool ephemeral = false;
};

}