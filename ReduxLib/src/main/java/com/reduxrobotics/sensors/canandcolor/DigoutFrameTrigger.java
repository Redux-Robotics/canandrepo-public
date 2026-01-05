// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor;

import com.reduxrobotics.frames.Frame;

/**
 * Enum representing whether to transmit a digout message when the overall value of a {@link DigoutChannel} changes state.
 * 
 * <p>Digout frames are by default transmitted every 100 ms on the CAN bus but using this enum
 * can cause the frame to be sent immidiately once the a digout channel's logic slots make its state change.</p>
 * 
 * <p>This can be used even if outputting digout state via the physical outputs is disabled and can be used to asynchronously
 * alert user code to some event via the {@link Frame} API, with the overarching goal of enabling robot code to react quickly 
 * to events.</p>
 * 
 * <p>
 * This enum can be passed to:
 * <ul>
 * <li>
 * {@link CanandcolorSettings#setDigoutFrameTrigger(DigoutChannel.Index, DigoutFrameTrigger)} which may be then used with 
 * {@link Canandcolor#setSettings(CanandcolorSettings)} to configure the device,
 * </li>
 * <li>
 * or to {@link DigoutChannel#configureFrameTrigger(DigoutFrameTrigger)}, which will directly configure the associated channel.
 * </li>
 * </ul>
 * 
 * As an applied example, consider the following code snippet:
 * 
 * <pre>
 * // Instantiate object
 * Canandcolor color = new Canandcolor(0); 
 * 
 * // Setup digouts; in this case we check if something is close.
 * color.digout1().configureSlots(new HSVDigoutConfig().setMaxProximity(0.4));
 * 
 * // This line configures digout packets to be sent immidiately when they change to true 
 * // but not when they change to false.
 * // 
 * // These messages are broadcast in addition to messages sent during regular frame periods.
 * color.setSettings(
 *   new CanandcolorSettings()
 *     .setDigoutFrameTrigger(DigoutChannel.Index.kDigout1, DigoutFrameTrigger.kFalling)
 * );
 * 
 * // This line does the same thing.
 * color.digout1().configureFrameTrigger(DigoutFrameTrigger.kFalling);
 * 
 * // Add a callback to perform some action
 * color.getDigoutFrame().addCallback(frame -> {
 *     // This callback gets executed on a separate message update thread away from robot update code.
 *     if (frame.getValue().getDigoutChannelValue(DigoutChannel.Index.kDigout1)) {
 *         // Perform some action; e.g. stopping an intake motor.
 *         // Note that this callback will fire on *any* digout frame, not just one that gets sent because of a message trigger,
 *         // as by default, digout messages are also broadcasted periodically.
 *         // User code is thus responsible for appropriate state management (and thread safety!) here.
 *     }
 * });
 * 
 * </pre>
 * 
 */
public enum DigoutFrameTrigger {

    /** Disable all extra digout logic-triggered frame packets for the digout channel. */
    kDisabled(0),
    /** Send a message when the digout channel value changes from false to true. */
    kRising(1),
    /** Send a message when the digout channel value changes from true to false. */
    kFalling(2),
    /** Send a message when the digout channle value changes at all. */
    kRisingAndFalling(3);

    private int index;
    DigoutFrameTrigger(int index) { this.index = index; }

    /**
     * Gets the corresponding index for the value in question.
     * @return the index for the opcode (used in serialization)
     */
    public int getIndex() { return index; }

    /**
     * Returns a corresponding vlaue from the given index.
     * @param idx the index to fetch.
     * @return a valid value. If invalid, returns disabled.
     */
    public static DigoutFrameTrigger fromIndex(int idx) {
        return switch (idx) {
            case 0 -> kDisabled;
            case 1 -> kRising;
            case 2 -> kFalling;
            case 3 -> kRisingAndFalling;
            default -> kDisabled;
        };
    }

}