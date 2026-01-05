// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once

// this header file exists to populate doxygen header files for each namespace that is used in more than one header file.
// including this does nothing otherwise.

/**
 * Top-level ReduxLib namespace
 */
namespace redux {

    /**
     * Namespace holding base and utility classes for all CAN-based Redux Robotics devices
     */
    namespace canand {};

    /**
     * Namespace holding all classes relating to Redux sensors
    */
    namespace sensors {};

    /**
     * Namespace holding classes relating to device data frames
    */
    namespace frames {};

}