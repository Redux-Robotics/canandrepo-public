// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor;

import java.util.Optional;

import com.reduxrobotics.canand.CanandSettingsManager;
import com.reduxrobotics.frames.Frame;

/**
 * Represents a digital output channel.
 * 
 * <p>
 * The {@link Canandcolor} has two digital output channels: {@link Canandcolor#digout1() digout1} 
 * and {@link Canandcolor#digout2() digout2}.
 * 
 * These channels have a boolean value that is computed by evaluating "digout slots" which can be configured via
 * {@link DigoutChannel#configureSlots(DigoutConfig)}.
 * These "digout slots" can be configured to evaluate boolean conditions on sensor values
 * to determine the overall channel's boolean value.
 * </p>
 * 
 * 
 * <p>
 * The value of these digout channels can be configued to be broadcasted over CAN or the associated 
 * physical 3.3v GPIO pins DIG-1 and DIG-2. 
 * The behavior of these GPIO pins can be configured using {@link #configureOutputPin(DigoutPinConfig)},
 * while the behavior of the digout frame can be configured using {@link CanandcolorSettings#setDigoutFramePeriod(double)}
 * to set the baseline frame period and {@link #configureFrameTrigger(DigoutFrameTrigger)} for behavior on state change.
 * </p>
 * 
 * <h2>Basic usage example</h2>
 * <pre>
 * Canandcolor color = new Canandcolor(0);
 * // Gets the digout1 channel
 * DigoutChannel digout1 = color.digout1();
 * 
 * // Check value over CAN
 * digout1.getValue();
 * // Check "sticky" value over CAN
 * digout1.getStickyValue();
 * 
 * // Check if the an object is close and the color sensor is seeing "blue" for at least 10 milliseconds,
 * // (Actual thresholds may need some experimental tuning, but will be in the same units as Canandcolor.getHue())
 * digout1.configureSlots(new HSVDigoutConfig()
 *   .setMaxProximity(0.15)
 *   .setProximityInRangeFor(0.01)
 *   .setMinHue(0.5)
 *   .setMaxHue(0.7)
 *   .setColorInRangeFor(0.01)
 * );
 * 
 * // Set the baseline CAN digout frame period to 50 ms.
 * // Note that this is global to both digout1 and digout2.
 * color.setSettings(new CanandcolorSettings().setDigoutFramePeriod(0.050));
 * 
 * // Instantly send CAN digout frames when the above condition changes at all.
 * digout1.configureFrameTrigger(DigoutFrameTrigger.kRisingAndFalling);
 * // Configure the DIG-1 GPIO pin to also output the condition
 * digout1.configureOutputPin(DigoutPinConfig.kDigoutLogicActiveHigh);
 * // Configure the DIG-2 GPIO pin to output proximity as a duty cycle
 * color.digout2().configureOutputPin(DataSource.kProximity);
 * </pre>
 * 
 */
public class DigoutChannel {
    private Canandcolor parent;
    private DigoutChannel.Index channelIndex;
    //final CanandSettingsManager<DigoutSettings> stg;
    final CanandSettingsManager<CanandcolorSettings> stg;
    DigoutChannel(Canandcolor parent, DigoutChannel.Index channelIndex) {
        this.parent = parent;
        this.stg = parent.getInternalSettingsManager();
        this.channelIndex = channelIndex;
    }

    /**
     * Returns the index of this digout channel.
     * @return channel index (either {@link DigoutChannel.Index#kDigout1} or {@link DigoutChannel.Index#kDigout2})
     */
    public DigoutChannel.Index channelIndex() {
        return channelIndex;
    }

    /**
     * Gets the current value of the digout channel, as reported over CAN.
     * <p>
     * By default, the digout frame period is a relatively slow 100ms, so if the most up-to-date
     * information is desired over CAN, changing this frame period via {@link CanandcolorSettings#setDigoutFramePeriod(double)}
     * or configuring transmission on state change via {@link #configureFrameTrigger(DigoutFrameTrigger)} may be desired.
     * </p>
     * @return digout channel value.
     */
    public boolean getValue() {
        long data = parent.getDigoutFrame().getData();
        if (channelIndex == DigoutChannel.Index.kDigout1) {
            return CanandcolorDetails.Msg.extractDigitalOutput_Digout1State(data);
        } else {
            return CanandcolorDetails.Msg.extractDigitalOutput_Digout2State(data);
        }
    }

    /**
     * Gets the current "sticky" value of the digout channel as reported over CAN.
     * <p>
     * This is similar to {@link #getValue()} except the "sticky" value will toggle to true
     * and will indefinitely stay true until {@link Canandcolor#clearStickyDigoutFlags()} is called.
     * </p>
     * <p>
     * This can be useful for catching when the digout channel goes to true, even if only for a few milliseconds.
     * </p>
     * 
     * @return sticky digout channel value.
     */
    public boolean getStickyValue() {
        long data = parent.getDigoutFrame().getData();
        if (channelIndex == DigoutChannel.Index.kDigout1) {
            return CanandcolorDetails.Msg.extractDigitalOutput_Digout1Sticky(data);
        } else {
            return CanandcolorDetails.Msg.extractDigitalOutput_Digout2Sticky(data);
        }
    }

    /**
     * Performs a low-level set on an indexed {@link DigoutSlot} on a Canandcolor digital output. 
     * <p><b>Most users should probably use {@link #configureSlots(DigoutConfig)} instead.</b></p>
     * 
     * <p>For details about how digout slots work and how they can detect sensor conditions at high speed, see {@link DigoutSlot} </p>
     * 
     * <p><b>These values persist on reboot!</b></p>
     * 
     * @param slotIndex The index of the slot to write to (0-15)
     * @param slotConfig The actual slot data (see {@link DigoutSlot})
     * @param timeout The timeout to wait for a confirmation message (set to 0 to not block)
     * @return whether or not the slot update was successful
     */
    boolean setRawDigoutSlot(int slotIndex, DigoutSlot slotConfig, double timeout) {
        if (slotIndex < 0 || slotIndex >= 16) { 
            throw new IllegalArgumentException("slot index must be between 0-15 inclusive"); 
        }
        int stgIdx = CanandcolorDetails.Stg.kDigout1Config0 - slotIndex;
        if (channelIndex == DigoutChannel.Index.kDigout2) {
            stgIdx -= 16;
        }

        return stg.confirmSetSetting(
            stgIdx,
            slotConfig.toSettingData(),
            timeout,
            0
        ).isValid();
    }

    /**
     * Performs a low-level set on an indexed {@link DigoutSlot} on a Canandcolor digital output with a 20 ms timeout. 
     * <p><b>Most users should probably use {@link #configureSlots(DigoutConfig)}</b></p>
     * 
     * <p>For details about how digout slots work and how they can detect sensor conditions at high speed, see {@link DigoutSlot} </p>
     * 
     * <p><b>These values persist on reboot!</b></p>
     * 
     * @param slotIndex The index of the slot to write to (0-15)
     * @param slotConfig The actual slot data (see {@link DigoutSlot})
     * @return whether or not the slot update was successful
     */
    boolean setRawDigoutSlot(int slotIndex, DigoutSlot slotConfig) {
        return setRawDigoutSlot(slotIndex, slotConfig, 0.10);
    }

    /**
     * Configures all the digout slots for a particular digital output.
     * 
     * <p><b>Most users should probably use {@link #configureSlots(DigoutConfig)}</b></p>
     * <p>For details about how digout slots work and how they can detect sensor conditions at high speed, see {@link DigoutSlot} </p>
     * 
     * <p>Note that if one particular slot write fails, 
     * the rest of the slot operations will still be attempted anyway.
     * This makes this method idempotent but not atomic.
     * </p>
     * 
     * @param triesPerSlot number of attempts to try to set each slot with
     * @param timeoutPerTry time in seconds to wait for timeout per try
     * @param chains the {@link DigoutChain}s to attempt to set the configuration from.
     * @return true on success, false on failure
     */
    boolean configureSlotsAdvanced(int triesPerSlot, double timeoutPerTry, DigoutChain... chains) {
        int len = 0;
        boolean success = true;
        for (DigoutChain c: chains) {
            len += c.length();
        }
        if (len > 16) {
            throw new IllegalArgumentException(String.format("Digout chains are too long (max: 16 slots, given: %d)", len));
        }

        int i = 0;
        for (DigoutChain c: chains) {
            for (int j = 0; j < c.length(); j++) {
                for (int attempt = 0; attempt < triesPerSlot; attempt++) {
                    if (setRawDigoutSlot(i, c.getSlot(j), timeoutPerTry)) {
                        break;
                    } else if (attempt == triesPerSlot - 1) {
                        success = false;
                    }
                }
                i++;
            }
        }
        // disable the rest of the slots
        DigoutSlot disabled = DigoutSlot.disabled();
        for (; i < 16; i++) {
            for (int attempt = 0; attempt < triesPerSlot; attempt++) {
                if (setRawDigoutSlot(i, disabled, timeoutPerTry)) {
                    break;
                } else if (attempt == triesPerSlot - 1) {
                    success = false;
                }
            }
        }

        return success;
    }

    /**
     * Configures all the digout slots for a particular digital output using {@link DigoutChain}s.
     * <p><b>Most users should probably use {@link #configureSlots(DigoutConfig)}</b></p>
     * 
     * <p>For details about how digout slots work and how they can detect sensor conditions at high speed, see {@link DigoutSlot} </p>
     * 
     * <pre>
     * // Instantiate object
     * Canandcolor color = new Canandcolor(0); 
     * 
     * // Setup digout1; in this case we check if something is close and if the red channel is bigger than the blue one.
     * color.digout1().configureSlotsAdvanced(DigoutChain.start()
     *   .lessThan(DataSource.kProximity, 0.1)
     *   .and()
     *   .greaterThan(DataSource.kRed, DataSource.kBlue)
     *   .finish()
     * );
     * 
     * // Get the boolean value from CAN
     * color.digout1().getValue();
     * </pre>
     * 
     * <p>Note that if one particular slot write fails, 
     * the rest of the slot operations will still be attempted anyway.
     * This makes this method idempotent but not atomic.
     * </p>
     * 
     * @param chains the {@link DigoutChain}s to attempt to set the configuration from.
     * @return true on success, false on failure
     */

    boolean configureSlotsAdvanced(DigoutChain... chains) {
        return configureSlotsAdvanced(3, 0.10, chains);
    }

    /**
     * Configures a digital output channel using the thresholds specified in a DigoutConfig.
     * 
     * <p>Available DigoutConfigs include:</p>
     * <ul>
     * <li>{@link RGBDigoutConfig} for using RGB color thresholds</li>
     * <li>{@link HSVDigoutConfig} for using HSV color thresholds</li>
     * </ul>
     * 
     * Basic usage example:
     * <pre>
     * Canandcolor color = new Canandcolor(0);
     * // Check if the an object is close and the color sensor is seeing blue for at least 10 milliseconds,
     * color.digout1().configureSlots(new HSVDigoutConfig()
     *   .setMaxProximity(0.15)
     *   .setProximityInRangeFor(0.01)
     *   .setMinHue(0.5)
     *   .setMaxHue(0.7)
     *   .setColorInRangeFor(0.01)
     * );
     * // Instantly send CAN digout frames when the above condition changes at all.
     * color.digout1().configureFrameTrigger(DigoutFrameTrigger.kRisingAndFalling);
     * // Configure the DIG-1 GPIO pin to also output the condition
     * color.digout1().configureOutputPin(DigoutPinConfig.kDigoutLogicActiveHigh);
     * // Check value over CAN
     * color.digout1().getValue();
     * </pre>
     * 
     * @param config The configuration to apply
     * @return true on success, false on failure. 
     *         This function is idempotent so calling it again until success will result in correct configuration.
     */
    public boolean configureSlots(DigoutConfig config) {
        return configureSlotsAdvanced(config.getDigoutChains());
    }

    /**
     * Configures a digital output channel using the thresholds specified in a DigoutConfig.
     * 
     * @param config The configuration to apply
     * @param triesPerSlot number of attempts to try to set each slot with (default 3)
     * @param timeoutPerTry time in seconds to wait for timeout per try (default 0.20)
     * @return true on success, false on failure. 
     *         This function is idempotent so calling it again until success will result in correct configuration.
     * @see #configureSlots(DigoutConfig)
     */
    public boolean configureSlots(DigoutConfig config, int triesPerSlot, double timeoutPerTry) {
        return configureSlotsAdvanced(triesPerSlot, timeoutPerTry, config.getDigoutChains());
    }

    /**
     * Configures the physical GPIO pin associated with the digout channel.
     * 
     * 
     * <p>Note that these pin outputs are independent of the actual digital output channel's value,
     * which is always continuously calcuated from digout slots.
     * </p>
     * 
     * <p>
     * These pins can be set into one of two or three modes:
     * </p>
     * 
     * <ul>
     * <li>output disabled, by passing in {@link DigoutPinConfig#kDisabled}</li> 
     * <li>output the value from the digout channel by passing in {@link DigoutPinConfig#kDigoutLogicActiveHigh}
     * or {@link DigoutPinConfig#kDigoutLogicActiveLow} </li>
     * <li>(only supported on {@link DigoutChannel.Index#kDigout2 kDigout2}) a duty cycle (PWM) output of values 
     * from either the color or proximity sensor, by passing in a {@link DataSource}) object </li>
     * </ul>
     * 
     * Some usage examples:
     * 
     * <pre>
     * Canandcolor color = new Canandcolor(0);
     * DigoutChannel digout2 = color.digout2();
     * 
     * // Disable the output pin
     * digout2.configureOutputPin(DigoutPinConfig.kDisabled);
     * 
     * // Use the digout slot value as the GPIO value, configure with a 20 ms timeout and up to 3 retries
     * digout2.configureOutputPin(DigoutPinConfig.kDigoutLogicActiveHigh, 0.1, 3);
     * 
     * // configure the pin to output proximity as a duty cycle value [0..1]
     * digout2.configureOutputPin(DataSource.kProximity);
     * 
     * // this will throw an IllegalArgumentException, as duty cycle is not supported on digout1.
     * color.digout1().configureOutputPin(DataSource.kProximity);
     * 
     * </pre>
     * 
     * @param config the {@link DigoutPinConfig} or {@link DataSource} to configure as output
     * @return true on known set success. This method attempts 3 tries at a 20 ms timeout.
     * @see CanandcolorSettings#setDigoutPinConfig(DigoutChannel.Index, DigoutPinConfig)
     */
    public boolean configureOutputPin(DigoutPinConfig config) {
        return configureOutputPin(config, 0.1, 3);
    }

    /**
     * Configures the associated physical GPIO pin associated with the digout channel.
     * <p>
     * See {@link #configureOutputPin(DigoutPinConfig)} for usage information.
     * </p>
     * @param config the {@link DigoutPinConfig} or {@link DataSource} to configure as output
     * @param timeout maximum timeout for this operation (in seconds). Set to zero to not check.
     * @param attempts the number of retry attempts to make the configuration succeed.
     * @return true on known set success.
     * @see #configureOutputPin(DigoutPinConfig)
     */
    public boolean configureOutputPin(DigoutPinConfig config, double timeout, int attempts) {
        if (config instanceof DataSource && channelIndex == DigoutChannel.Index.kDigout1) {
            throw new IllegalArgumentException("Digout 1 does not support duty cycle GPIO output!");
        }

        int idx = (channelIndex == DigoutChannel.Index.kDigout1) 
                  ? CanandcolorDetails.Stg.kDigout1OutputConfig
                  : CanandcolorDetails.Stg.kDigout2OutputConfig;
        boolean success = false;
        for (int i = 0; i < attempts && !success; i++) {
            success = stg.confirmSetSetting(
                idx,
                config.toOutputSettingData(),
                timeout,
                0
            ).isValid();
        }
        return success;
    }

    /**
     * Configures the frame trigger settings for this digout channel.
     * <p>These determine if extra digout frames should get sent if the current digout channel
     * changes state. These can be used with the {@link Frame} interface to run callbacks
     * on state change, or simply ensure that {@link #getValue()} always has an up-to-date value.
     * </p>
     * 
     * <p>This will increase CAN usage -- potentially dramatically so if the digout slot conditions
     * have been misconfigured to toggle rapidly. Using time-based conditions such as
     * {@link HSVDigoutConfig#setProximityInRangeFor(double)} or {@link DigoutSlotBuilder#trueFor(double)}
     * can "debounce" digout conditions preventing this from happening.
     * </p>
     * 
     * @param trigger the {@link DigoutFrameTrigger} to set
     * @return true on set success
     * @see CanandcolorSettings#setDigoutFrameTrigger(DigoutChannel.Index, DigoutFrameTrigger)
     */
    public boolean configureFrameTrigger(DigoutFrameTrigger trigger) {
        return configureFrameTrigger(trigger, 0.1, 3);
    }

    /**
     * Configures the frame trigger settings for this digout channel.
     * <p>See {@link #configureFrameTrigger(DigoutFrameTrigger)} for usage information.</p>
     * @param trigger the {@link DigoutFrameTrigger} to set
     * @param timeout maximum timeout for this operation (in seconds)
     * @param attempts the number of retry attempts to make the configuration succeed.
     * @return true on known set success. This method attempts 3 tries at a 20 ms timeout.
     * @see #configureFrameTrigger(DigoutFrameTrigger)
     */
    public boolean configureFrameTrigger(DigoutFrameTrigger trigger, double timeout, int attempts) {
        int idx = (channelIndex == DigoutChannel.Index.kDigout1) 
                  ? CanandcolorDetails.Stg.kDigout1MessageOnChange
                  : CanandcolorDetails.Stg.kDigout2MessageOnChange;

        boolean success = false;
        for (int i = 0; i < attempts && !success; i++) {
            success = stg.confirmSetSetting(
                idx,
                trigger.getIndex(),
                timeout,
                0
            ).isValid();
        }
        return success;

    }

    /**
     * Fetches a digout slot's configuration.
     * <p><b>Most users will not need to use this method.</b></p>
     * <p>For information about how these work, see {@link DigoutSlot}.</p>
     * 
     * @param slotIndex The index of the slot to fetch from (0-15)
     * @param timeout The timeout to wait for the slot to be retrieved (recommend 0.1)
     * @return DigoutSlot object on success, {@link Optional#empty} on failure
     */
    Optional<DigoutSlot> getRawDigoutSlot(int slotIndex, double timeout) {
        if (slotIndex < 0 || slotIndex >= 16) { 
            throw new IllegalArgumentException("slot index must be between 0-15 inclusive"); 
        }
        int stgIdx = CanandcolorDetails.Stg.kDigout1Config0 - slotIndex;
        if (channelIndex == DigoutChannel.Index.kDigout2) {
            stgIdx -= 16;
        }

        CanandSettingsManager.SettingResult res = stg.fetchSetting(stgIdx, timeout);
        if (!res.isValid()) return Optional.empty();
        return Optional.of(DigoutSlot.fromSettingData(res.value()));
    }
    /**
     * Fetches a digout slot's configuration with a default 20 ms timeout.
     * <p><b>Most users will not need to use this method.</b></p>
     * <p>For information about how these work, see {@link DigoutSlot}.</p>
     * 
     * @param slotIndex The index of the slot to fetch from (0-15)
     * @return DigoutSlot object on success, {@link Optional#empty} on failure
     */
    Optional<DigoutSlot> getRawDigoutSlot(int slotIndex) {
        return getRawDigoutSlot(slotIndex, 0.10);
    }
    
    /**
     * Clears all configured "slots" on the specified digital output, setting them all to be disabled.
     * 
     * <p>This clears any configuration previously set by {@link #configureSlots(DigoutConfig)}.
     */
    public void clearAllDigoutSlots() {
        stg.sendSettingCommand(switch(channelIndex) {
            case kDigout1 -> CanandcolorDetails.Enums.SettingCommand.kClearDigout1;
            case kDigout2 -> CanandcolorDetails.Enums.SettingCommand.kClearDigout2;
            // unreachable
            default -> CanandcolorDetails.Enums.SettingCommand.kClearDigout1;
        });
    }

    /** 
     * Enum representing the index of a {@link DigoutChannel digital output channel} on a {@link Canandcolor}.
     * @see DigoutChannel
     */
    public enum Index {
        /** The first digital output channel. */
        kDigout1,
        /** The second digital output channel. */
        kDigout2;
    }

}
