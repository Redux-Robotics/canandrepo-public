// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandmag;

/**
 * Contains all the constants used by the {@link Canandmag} and related classes (as to not pollute their public namespaces)
 */
public class CanandmagDetails {
    // non-instantiable class
    private CanandmagDetails() {}
    // ------ CAN API ids ------

    /** Message id for Position frame */
    public static final int kMsg_PositionOutput      = 0x1f;
    /** Message id for Velocity frame */
    public static final int kMsg_VelocityOutput      = 0x1e;
    /** Message id for Raw position frame */
    public static final int kMsg_RawPositionOutput   = 0x1d;

    // Inherited from CanandDevice
    /** Message id for setting control command */
    public static final int kMsg_SettingCommand                 = 0x2;
    /** Message id for update setting on device */
    public static final int kMsg_SetSetting                     = 0x3;
    /** Message id for setting value report from device */
    public static final int kMsg_ReportSetting                  = 0x4;
    /** Message id for clear device sticky faults */
    public static final int kMsg_ClearStickyFaults              = 0x5;
    /** Message id for status frames */
    public static final int kMsg_Status                         = 0x6;
    /** Message id for party mode */
    public static final int kMsg_PartyMode                      = 0x7;

    // ------ Setting indexes ------
    /** setting id for Encoder zero offset */
    public static final int kStg_ZeroOffset               = 0xff;
    /** setting id for Velocity window width (value*250us) */
    public static final int kStg_VelocityWindow           = 0xfe;
    /** setting id for Position frame period (ms) */
    public static final int kStg_PositionFramePeriod      = 0xfd;
    /** setting id for Velocity frame period (ms) */
    public static final int kStg_VelocityFramePeriod      = 0xfc;
    /** setting id for Raw position frame period (ms) */
    public static final int kStg_RawPositionFramePeriod   = 0xfb;
    /** setting id for Invert direction (use cw instead of ccw) */
    public static final int kStg_InvertDirection          = 0xfa;
    /** setting id for Relative position value */
    public static final int kStg_RelativePosition         = 0xf9;
    /** setting id for Disable the zero button */
    public static final int kStg_DisableZeroButton        = 0xf8;

    // Inherited from CanandDevice
    /** setting id for status frame period (ms) */
    public static final int kStg_StatusFramePeriod        = 0x4;

    // ----- Setting commands ------
    /** setting command for Factory defaults, but keep the encoder zero offset */
    public static final int kStgCmd_ResetFactoryDefaultKeepZero  = 0xff;

    // Inherited from CanandDevice
    /** setting command for Fetch all settings from device */
    public static final int kStgCmd_FetchSettings          = 0x0;
    /** setting command for Reset everything to factory default */
    public static final int kStgCmd_ResetFactoryDefault    = 0x1;
    /** setting command for Fetch individual setting */
    public static final int kStgCmd_FetchSettingValue      = 0x2;

}
