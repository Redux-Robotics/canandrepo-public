// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include "redux/sensors/canandgyro/CanandgyroFaults.h"
#include "redux/sensors/canandgyro/CanandgyroData.h"
#include "redux/sensors/canandgyro/CanandgyroStatus.h"
#include "wpi/struct/Struct.h"

namespace wpi {

/** WPILib struct template for CanandgyroFaults */
template <>
struct Struct<::redux::sensors::canandgyro::CanandgyroFaults> {
    /** @return Type string name */
    static constexpr std::string_view GetTypeName() {
        return "CanandgyroFaults";
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
            "bool calibrating:1;"
            "bool angular_velocity_saturation:1;"
            "bool acceleration_saturation:1;"
        );
    }

    /** 
     * @param data to unpack to object
     * @return CanandgyroFaults
     */
    static ::redux::sensors::canandgyro::CanandgyroFaults Unpack(std::span<const uint8_t> data);
    /**
     * @param data to pack object into
     * @param value object reference
     */
    static void Pack(std::span<uint8_t> data,
                     const ::redux::sensors::canandgyro::CanandgyroFaults& value);
};

/** WPILib struct template for CanandgyroStatus */
template <>
struct Struct<::redux::sensors::canandgyro::CanandgyroStatus> {
    /** @return Type string name */
    static constexpr std::string_view GetTypeName() {
        return "CanandgyroStatus";
    }
    /** @return size of serialized payload in bytes */
    static constexpr size_t GetSize() { return 10; }
    /** @return wpistruct schema */
    static constexpr std::string_view GetSchema() {
        return (
            "CanandgyroFaults active_faults;"
            "CanandgyroFaults sticky_faults;"
            "double temperature;"
        );
    }

    /** 
     * @param data to unpack to object
     * @return CanandgyroStatus
     */
    static ::redux::sensors::canandgyro::CanandgyroStatus Unpack(std::span<const uint8_t> data);
    /**
     * @param data to pack object into
     * @param value object reference
     */
    static void Pack(std::span<uint8_t> data,
                     const ::redux::sensors::canandgyro::CanandgyroStatus& value);
    /**
     * @param fn to run 
     */
    static void ForEachNested(
        std::invocable<std::string_view, std::string_view> auto fn) {
     wpi::ForEachStructSchema<::redux::sensors::canandgyro::CanandgyroFaults>(fn);
    }
};

/** WPILib struct template for Canandgyro AngularVelocity */
template <>
struct Struct<::redux::sensors::canandgyro::AngularVelocity> {
    /** @return Type string name */
    static constexpr std::string_view GetTypeName() {
        return "CanandgyroAngularVelocity";
    }
    /** @return size of serialized payload in bytes */
    static constexpr size_t GetSize() { return 24; }
    /** @return wpistruct schema */
    static constexpr std::string_view GetSchema() {
        return (
            "double roll;"
            "double pitch;"
            "double yaw;"
        );
    }

    /** 
     * @param data to unpack to object
     * @return object
     */
    static ::redux::sensors::canandgyro::AngularVelocity Unpack(std::span<const uint8_t> data);
    /**
     * @param data to pack object into
     * @param value object reference
     */
    static void Pack(std::span<uint8_t> data,
                     const ::redux::sensors::canandgyro::AngularVelocity& value);
};

/** WPILib struct template for Canandgyro Acceleration */
template <>
struct Struct<::redux::sensors::canandgyro::Acceleration> {
    /** @return Type string name */
    static constexpr std::string_view GetTypeName() {
        return "CanandgyroAcceleration";
    }
    /** @return size of serialized payload in bytes */
    static constexpr size_t GetSize() { return 24; }
    /** @return wpistruct schema */
    static constexpr std::string_view GetSchema() {
        return (
            "double x;"
            "double y;"
            "double z;"
        );
    }

    /** 
     * @param data to unpack to object
     * @return object
     */
    static ::redux::sensors::canandgyro::Acceleration Unpack(std::span<const uint8_t> data);
    /**
     * @param data to pack object into
     * @param value object reference
     */
    static void Pack(std::span<uint8_t> data,
                     const ::redux::sensors::canandgyro::Acceleration& value);
};
}


static_assert(wpi::StructSerializable<::redux::sensors::canandgyro::AngularVelocity>);
static_assert(wpi::StructSerializable<::redux::sensors::canandgyro::Acceleration>);
static_assert(wpi::StructSerializable<::redux::sensors::canandgyro::CanandgyroStatus>);
static_assert(wpi::StructSerializable<::redux::sensors::canandgyro::CanandgyroFaults>);
static_assert(wpi::HasNestedStruct<::redux::sensors::canandgyro::CanandgyroStatus>);