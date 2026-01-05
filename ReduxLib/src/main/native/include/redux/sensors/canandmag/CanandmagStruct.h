// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include "redux/sensors/canandmag/CanandmagFaults.h"
#include "redux/sensors/canandmag/CanandmagStatus.h"
#include "wpi/struct/Struct.h"

namespace wpi {

/** WPILib struct template for CanandmagFaults */
template <>
struct Struct<::redux::sensors::canandmag::CanandmagFaults> {
    /** @return Type string name */
    static constexpr std::string_view GetTypeName() {
        return "CanandmagFaults";
    }
    /** @return size of serialized payload in bytes */
    static constexpr size_t GetSize() { return 1; }

    /** @return wpistruct schema */
    static constexpr std::string_view GetSchema() {
        return (
            "bool power_cycle:1;"
            "bool can_id_conflict:1;"
            "bool can_general_error:1;"
            "bool out_of_temperature_range:1;"
            "bool hardware_fault:1;"
            "bool magnet_out_of_range:1;"
            "bool under_volt:1;"
            "bool reserved:1;"
        );
    }

    /** 
     * @param data to unpack to object
     * @return CanandmagFaults
     */
    static ::redux::sensors::canandmag::CanandmagFaults Unpack(std::span<const uint8_t> data);
    /**
     * @param data to pack object into
     * @param value object reference
     */
    static void Pack(std::span<uint8_t> data,
                     const ::redux::sensors::canandmag::CanandmagFaults& value);
};

/** WPILib struct template for CanandmagStatus */
template <>
struct Struct<::redux::sensors::canandmag::CanandmagStatus> {
    /** @return Type string name */
    static constexpr std::string_view GetTypeName() {
        return "CanandmagStatus";
    }
    /** @return size of serialized payload in bytes */
    static constexpr size_t GetSize() { return 10; }
    /** @return wpistruct schema */
    static constexpr std::string_view GetSchema() {
        return (
            "CanandmagFaults active_faults;"
            "CanandmagFaults sticky_faults;"
            "double temperature;"
        );
    }

    /** 
     * @param data to unpack to object
     * @return CanandmagStatus
     */
    static ::redux::sensors::canandmag::CanandmagStatus Unpack(std::span<const uint8_t> data);
    /**
     * @param data to pack object into
     * @param value object reference
     */
    static void Pack(std::span<uint8_t> data,
                     const ::redux::sensors::canandmag::CanandmagStatus& value);
    /**
     * @param fn to run 
     */
    static void ForEachNested(
        std::invocable<std::string_view, std::string_view> auto fn) {
     wpi::ForEachStructSchema<::redux::sensors::canandmag::CanandmagFaults>(fn);
    }
};
}

static_assert(wpi::StructSerializable<::redux::sensors::canandmag::CanandmagStatus>);
static_assert(wpi::StructSerializable<::redux::sensors::canandmag::CanandmagFaults>);
static_assert(wpi::HasNestedStruct<::redux::sensors::canandmag::CanandmagStatus>);