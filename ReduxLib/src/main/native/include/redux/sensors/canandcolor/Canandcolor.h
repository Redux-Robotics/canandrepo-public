// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <units/time.h>
#include "redux/canand/CanandDevice.h"
#include "redux/frames/Frame.h"
#include "redux/canand/CanandSettingsManager.h"
#include "CanandcolorDetails.h"
#include "CanandcolorStatus.h"
#include "CanandcolorFaults.h"
#include "CanandcolorSettings.h"
#include "ColorData.h"
#include "DigoutSlotState.h"

/**
 * Namespace for all classes relating to the Canandcolor.
 */
namespace redux::sensors::canandcolor {

/**
 * Class for the CAN interface of the
 * <a href="https://docs.reduxrobotics.com/canandcolor/index.html">Canandcolor.</a>
 *
 * <p>
 * The C++ vendordep uses the
 * <a href="https://docs.wpilib.org/en/stable/docs/software/basic-programming/cpp-units.html">WPILib units library</a>
 * for all dimensioned values, including settings.
 * </p>
 *
 * <p>
 * Sensor reads (proximity/color/status) do not block: data is received asynchronously from the CAN receive thread and
 * getters return the most recently received values.
 * </p>
 *
 * <p>
 * Settings operations may block briefly (default ~20 ms per setting) while waiting for a confirmation packet, unless
 * the timeout is set to 0.
 * </p>
 *
 * Example code:
 * ```cpp
 * Canandcolor color{0}; // device with CAN id 0
 *
 * // Non-blocking reads (last received values):
 * double proximity = color.GetProximity(); // normalized [0..1]
 * double r = color.GetRed();              // normalized [0..1]
 * double hue = color.GetHSVHue();         // normalized [0..1)
 *
 * // Timestamped reads:
 * auto proximityFrame = color.GetProximityFrame().GetFrameData();
 * proximityFrame.GetValue();
 * proximityFrame.GetTimestamp();
 *
 * // Settings (blocking):
 * CanandcolorSettings settings;
 * settings.SetStatusFramePeriod(1000_ms);
 * settings.SetProximityFramePeriod(20_ms);
 * settings.SetColorFramePeriod(20_ms);
 * settings.SetColorIntegrationPeriod(ColorPeriod::k16ms);
 * settings.SetProximityIntegrationPeriod(ProximityPeriod::k25ms);
 * color.SetSettings(settings, 20_ms);
 *
 * // LED control:
 * color.SetLampLEDBrightness(1.0); // clamp to [0..1]
 *
 * // Faults:
 * color.ClearStickyFaults();
 * auto sticky = color.GetStickyFaults();
 * sticky.powerCycle;
 *
 * // Digital output config:
 * settings.SetDigoutPinConfig(0, ActiveHighDigoutPinConfig{});
 * settings.SetDigoutFrameTrigger(0, DigoutFrameTrigger::kRisingAndFalling);
 * ```
 */
class Canandcolor : public redux::canand::CanandDevice {
 public:
  /**
   * Constructor with the device's CAN id.
   *
   * This object will be constant with respect to whatever CAN id assigned to it, so if a device changes id it may
   * change which device this object reads from.
   *
   * @param canID device CAN id [0..63]
   * @param bus the message bus to use. Defaults to "halcan".
   */
  Canandcolor(int canID, std::string bus = "halcan");

  /**
   * Destructor.
   *
   * Unregisters the internal CAN listener.
   */
  ~Canandcolor();

  /**
   * Gets the most recently received proximity value, normalized to [0..1].
   *
   * @return proximity [0..1]
   */
  double GetProximity();

  /**
   * Gets the most recently received red channel, normalized to [0..1].
   *
   * @return red [0..1]
   */
  double GetRed();

  /**
   * Gets the most recently received green channel, normalized to [0..1].
   *
   * @return green [0..1]
   */
  double GetGreen();

  /**
   * Gets the most recently received blue channel, normalized to [0..1].
   *
   * @return blue [0..1]
   */
  double GetBlue();

  /**
   * Gets the HSV hue derived from the most recently received RGB channels.
   *
   * The hue is normalized to [0..1), where 0 and 1 represent the same hue.
   *
   * @return hue [0..1)
   */
  double GetHSVHue();

  /**
   * Gets the HSV saturation derived from the most recently received RGB channels.
   *
   * @return saturation [0..1]
   */
  double GetHSVSaturation();

  /**
   * Gets the HSV value derived from the most recently received RGB channels.
   *
   * @return value [0..1]
   */
  double GetHSVValue();

  /**
   * Gets the most recently received RGB triplet.
   *
   * @return ColorData snapshot of the most recently received color
   */
  ColorData GetColor();

  /**
   * Gets the most recently received digital output state.
   *
   * @return DigoutSlotState snapshot of the most recently received digital output frame
   */
  DigoutSlotState GetDigoutState();

  /**
   * Gets sticky faults from the most recently received status frame.
   *
   * Sticky faults remain set until cleared via ClearStickyFaults().
   *
   * @return sticky faults (faultsValid indicates whether the device has reported status yet)
   */
  CanandcolorFaults GetStickyFaults();

  /**
   * Gets active faults from the most recently received status frame.
   *
   * @return active faults (faultsValid indicates whether the device has reported status yet)
   */
  CanandcolorFaults GetActiveFaults();

  /**
   * Gets the most recently received temperature.
   *
   * @return temperature in degrees Celsius
   */
  units::celsius_t GetTemperature();

  /**
   * Gets the most recently received device status.
   *
   * @return CanandcolorStatus snapshot
   */
  CanandcolorStatus GetStatus();

  /**
   * Clears sticky faults on the device.
   *
   * This call does not block.
   */
  void ClearStickyFaults();

  /**
   * Clears sticky digital output event flags on the device.
   *
   * This call does not block.
   */
  void ClearStickyDigoutFlags();

  /**
   * Controls "party mode" (device identification LED animation).
   *
   * This call does not block.
   *
   * @param level party level, clamped to [0..10]
   */
  void SetPartyMode(uint8_t level);

  /**
   * Fetches the device settings in a blocking manner.
   *
   * @param timeout total timeout to wait for all settings
   * @param missingTimeout per-setting timeout for retrying missing settings
   * @param attempts number of retries for missing settings
   * @return received settings snapshot
   */
  CanandcolorSettings GetSettings(units::second_t timeout = 350_ms,
                                   units::second_t missingTimeout = 20_ms,
                                   uint32_t attempts = 3);

  /**
   * Starts an asynchronous fetch of settings.
   *
   * Use GetSettingsAsync() to retrieve whatever is currently known.
   */
  void StartFetchSettings();

  /**
   * Gets the currently known settings cache.
   *
   * @return settings currently received so far
   */
  CanandcolorSettings GetSettingsAsync();

  /**
   * Sets device settings.
   *
   * @param settings settings to apply
   * @param timeout maximum time to wait for each setting to be confirmed; set to 0 to not block
   * @param attempts number of attempts per setting
   * @return settings object containing any settings that failed to apply
   */
  CanandcolorSettings SetSettings(CanandcolorSettings& settings,
                                    units::second_t timeout = 20_ms,
                                    uint32_t attempts = 3);

  /**
   * Resets the device to factory defaults.
   *
   * @param timeout total timeout to wait for the reset command to be confirmed
   * @return received settings snapshot after reset (best-effort)
   */
  CanandcolorSettings ResetFactoryDefaults(units::second_t timeout = 350_ms);

  /**
   * Sets the lamp LED brightness.
   *
   * @param brightness desired brightness, clamped to [0..1]
   */
  void SetLampLEDBrightness(double brightness);

  /**
   * Gets the proximity frame, which holds the most recently received proximity value and its receive timestamp.
   *
   * @return reference to the internal Frame<double>
   */
  inline redux::frames::Frame<double>& GetProximityFrame() {
    return proximity_;
  }

  /**
   * Gets the color frame, which holds the most recently received ColorData and its receive timestamp.
   *
   * @return reference to the internal ColorFrame
   */
  inline redux::frames::Frame<ColorData>& GetColorFrame() {
    return color_;
  }

  /**
   * Gets the digital output frame, which holds the most recently received DigoutSlotState and its receive timestamp.
   *
   * @return reference to the internal Frame<DigoutSlotState>
   */
  inline redux::frames::Frame<DigoutSlotState>& GetDigoutFrame() {
    return digout_;
  }

  /**
   * Gets the status frame, which holds the most recently received CanandcolorStatus and its receive timestamp.
   *
   * @return reference to the internal Frame<CanandcolorStatus>
   */
  inline redux::frames::Frame<CanandcolorStatus>& GetStatusFrame() {
    return status_;
  }

  /**
   * Gets the internal settings manager.
   *
   * This is an advanced API for custom settings workflows.
   *
   * @return internal CanandSettingsManager handle
   */
  inline redux::canand::CanandSettingsManager<CanandcolorSettings>& GetInternalSettingsManager() {
    return stg_;
  }

  /**
   * Internal CAN message handler.
   * @param msg received CAN message
   */
  void HandleMessage(redux::canand::CanandMessage& msg) override;

  /**
   * Gets the CAN device address for this instance.
   * @return CanandAddress reference
   */
  redux::canand::CanandAddress& GetAddress() override;

  /**
   * Gets the device class name string.
   * @return "Canandcolor"
   */
  std::string GetDeviceClassName() override;

  /**
   * Gets the minimum firmware version expected by this library.
   * @return minimum firmware version
   */
  redux::canand::CanandFirmwareVersion GetMinimumFirmwareVersion() override;

 protected:
  /** proximity frame */
  redux::frames::Frame<double> proximity_;
  /** color data frame */
  redux::frames::Frame<ColorData> color_;
  /** digout status frame */
  redux::frames::Frame<DigoutSlotState> digout_;
  /** status frame*/
  redux::frames::Frame<CanandcolorStatus> status_;
  /** stg manager */
  redux::canand::CanandSettingsManager<CanandcolorSettings> stg_;

 private:
  redux::canand::CanandAddress addr_;
  bool dataRecvOnce_;
  units::second_t lastMessageTime_;
};

}  // namespace redux::sensors::canandcolor
