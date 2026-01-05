// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once

#include <cinttypes>
#include <algorithm>
#include <Eigen/Core>
#include <units/force.h>
#include <units/angular_velocity.h>

namespace redux::sensors::canandgyro {

/**
 * Angular velocity data class.
*/
class AngularVelocity {
  public:
  /** constructor 
   * @param roll roll
   * @param pitch pitch
   * @param yaw yaw
   * 
  */
    constexpr AngularVelocity(
        const units::turns_per_second_t roll, 
        const units::turns_per_second_t pitch,
        const units::turns_per_second_t yaw
    ) : roll{roll}, pitch{pitch}, yaw{yaw} {};

    /** 
     * Roll velocity.
     * @return roll velocity in angular velocity units
     */
    constexpr units::turns_per_second_t Roll() const { return this->roll; }

    /** 
     * Pitch velocity.
     * @return pitch velocity in angular velocity units
     */
    constexpr units::turns_per_second_t Pitch() const { return this->pitch; }

    /** 
     * Yaw velocity.
     * @return yaw velocity in angular velocity units
     */
    constexpr units::turns_per_second_t Yaw() const { return this->yaw; }

    /**
     * Converts to a Eigen::Vector3d with roll/pitch/yaw velocity as the first/second/third element 
     * and the units in radians/second.
     * @return Vector3d
     */
    inline Eigen::Vector3d ToVector3d() {
        return Eigen::Vector3d {
            this->roll.convert<units::radians_per_second>().value(),
            this->pitch.convert<units::radians_per_second>().value(),
            this->yaw.convert<units::radians_per_second>().value(),
        };
    }


  private:
    units::turns_per_second_t roll;
    units::turns_per_second_t pitch;
    units::turns_per_second_t yaw;

};


/**
 * Acceleration data class.
*/
class Acceleration {
  public:
  /** 
   * constructor 
   * @param x x-axis
   * @param y y-axis
   * @param z z-axis
   */
    constexpr Acceleration(
        const units::standard_gravity_t x,
        const units::standard_gravity_t y,
        const units::standard_gravity_t z
    ) : x{x}, y{y}, z{z} {};

    /** 
     * X-axis component
     * @return x in accelerational units
     */
    constexpr units::standard_gravity_t X() const { return this->x; }
    /**
     * Y-axis component
     * @return y in accelerational units
     */
    constexpr units::standard_gravity_t Y() const { return this->y; }
    /**
     * Z-axis component
     * @return z in accelerational units
     */
    constexpr units::standard_gravity_t Z() const { return this->z; }

    /**
     * Converts to a Eigen Vector3d with X/Y/Z-axis acceleration as the first/second/third element 
     * and standard gravities as the unit.
     * @return a Vector3d
     */
    inline Eigen::Vector3d ToVector3d() {
        return Eigen::Vector3d {
            this->x.value(),
            this->y.value(),
            this->z.value()
        };
    }


  private:
    units::standard_gravity_t x;
    units::standard_gravity_t y;
    units::standard_gravity_t z;

};

}