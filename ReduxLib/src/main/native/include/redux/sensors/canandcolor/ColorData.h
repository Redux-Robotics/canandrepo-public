// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <algorithm>
#include <cmath>

namespace redux::sensors::canandcolor {

/**
 * Normalized RGB color reading.
 *
 * All channels are normalized to [0..1]. HSV helper functions also return normalized values:
 * - Hue in [0..1)
 * - Saturation in [0..1]
 * - Value in [0..1]
 */
struct ColorData {
  /**
   * Color data struct
   * @param red value [0..1]
   * @param green value [0..1]
   * @param blue value [0..1]
   */
  constexpr ColorData(double red, double green, double blue) :
      red{red}, green{green}, blue{blue} {};

  /** red value [0..1] */
  double red;
  /** green value [0..1] */
  double green;
  /** blue value [0..1] */
  double blue;

  /**
   * Gets normalized HSV hue derived from this RGB value.
   * @return hue in [0..1)
   */
  constexpr double GetHSVHue() const {
    return HSVHue(red, green, blue);
  }

  /**
   * Gets normalized HSV saturation derived from this RGB value.
   * @return saturation in [0..1]
   */
  constexpr double GetHSVSaturation() const {
    return HSVSaturation(red, green, blue);
  }

  /**
   * Gets normalized HSV value derived from this RGB value.
   * @return value in [0..1]
   */
  constexpr double GetHSVValue() const {
    return HSVValue(red, green, blue);
  }

  /**
   * Constructs a ColorData value from a packed CAN color message payload.
   * @param data packed message data (see protocol details)
   * @return ColorData normalized to [0..1]
   */
  static constexpr ColorData FromColorMessage(details::msg::ColorOutput data) {
    constexpr double FACTOR = 1.0 / ((1 << 20) - 1);
    return ColorData{data.red * FACTOR, data.green * FACTOR, data.blue * FACTOR};
  }

  /**
   * Computes normalized HSV hue from normalized RGB.
   * @param r red in [0..1]
   * @param g green in [0..1]
   * @param b blue in [0..1]
   * @return hue in [0..1)
   */
  static constexpr double HSVHue(double r, double g, double b) {
    double maxVal = std::max(r, std::max(g, b));
    double minVal = std::min(r, std::min(g, b));
    double chroma = maxVal - minVal;

    if (chroma == 0.0) return 0.0;
    if (maxVal == r) { return std::fmod(((g - b) / chroma), 6.0) / 6.0; }
    if (maxVal == g) { return (((b - r) / chroma) + 2) / 6.0; }
    if (maxVal == b) { return (((r - g) / chroma) + 4) / 6.0; }
    return 0.0;
  }

  /**
   * Computes normalized HSV saturation from normalized RGB.
   * @param r red in [0..1]
   * @param g green in [0..1]
   * @param b blue in [0..1]
   * @return saturation in [0..1]
   */
  static constexpr double HSVSaturation(double r, double g, double b) {
    double maxVal = std::max(r, std::max(g, b));
    double minVal = std::min(r, std::min(g, b));

    if (maxVal == 0) return 0;
    return (maxVal - minVal) / maxVal;
  }

  /**
   * Computes normalized HSV value from normalized RGB.
   * @param r red in [0..1]
   * @param g green in [0..1]
   * @param b blue in [0..1]
   * @return value in [0..1]
   */
  static constexpr double HSVValue(double r, double g, double b) {
    return std::max(r, std::max(g, b));
  }
};

}  // namespace redux::sensors::canandcolor
