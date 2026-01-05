.. _msg_position_output:

Periodic frame with relative position and absolute position registers read by the device.

Absolute and relative position operate (mostly) independently; 
absolute position has a persistent zero offset that is subtracted from the raw zero reading and is limited to a single rotation 
whereas relative position always initializes to 0 on boot and counts across multiple turns.

Both absolute and relative position have their outputs set to 0 when the zero button is pressed for 2 seconds,
and both absolute and relative position will always count up and count down in the same direction at the same rate.

The absolute position value is also what gets sent over PWM, and so will also be affected by the zero offset.

For both relative (unwrapped) and absolute position, one device tick is 1/16384-th of a rotation.

.. _msg_velocity_output:

Periodic frame containing the currently calculated velocity.

.. _msg_raw_position_output:

Periodic frame containing a raw absolute reading that does not account for the zero offset or inversion settings.
Additionally includes a reading timestamp in milliseconds and magnet status data.

**By factory default, this frame is disabled (0 ms) -- it needs to be explicitly enabled to be used.**

.. _msg_status:

Periodic status frame containing active and sticky fault data as well as temperature.

.. _setting_position_frame_period:

:frame_period:msg_position_output

.. _setting_velocity_frame_period:

:frame_period:msg_velocity_output

.. _setting_raw_position_frame_period:

:frame_period:msg_raw_position_output

.. _setting_invert_direction:

This setting whether counter clockwise or clockwise relative to the Canandmag's sensor face (the side opposite its LED) 
should be positive for position (both relative and absolute) and velocity.

This will additionally invert the direction of the absolute PWM output as well.

False (0) means counter clockwise is positive, true (1) specifies clockwise is positive.

This setting affects the direction of both absolute and relative position.

.. _setting_zero_offset:

This setting is used to update the absolute position broadcasted by the position message and over PWM.

Absolute position is calculated as:

.. math::

    \text{absolute position} = (\text{raw IC reading} - \text{saved zero offset}) \mod (1 rotation)

with some additional logic to handle direction inversion.

There are two methods to updating the absolute position: 

* updating the zero offset (saved in flash) subtracted from the raw encoder IC reading directly
* taking a new intended absolute encoder reading and calculating the zero offset required to read the new value

The former has some niche uses but the latter is by far the most common application (e.g. to zero the encoder position).

This setting can update the zero offset via either approach via the position bit. 
If the position bit is set, the sent value is treated as a new absolute position, but if unset, the sent value is treated as a new zero offset.

If this setting is read from, the zero offset is always returned with the position bit unset.


.. _setting_relative_position:

This setting can be used to update the value of the relative position broadcasted 
by the :ref:`periodic position frame<msg_position_output>`. 

This current position register does not have an offset saved to non-volatile flash but is rather initialized to 0 on device boot and
updated as the encoder IC reading changes.

Writing to this settings with a signed 32-bit integer will update the current relative position to the new value in the payload, 
using the lowest 32 bits as a signed integer.

Reading from this settings index is not valid and will produce 0xffffffffffff -- use 
the :ref:`periodic position frame<msg_position_output>` to read relative position data instead.

.. _setting_disable_zero_button:

This setting controls whether the onboard zero button functions.

By default, this setting is set to false (0). Pressing the zero button for 2 seconds will set both absolute and relative position values

When the zero button is enabled (when this setting is set to 0), pressing it for 2 seconds will set both absolute and relative position 
values to zero for both CAN and PWM, and pressing the button for 15 seconds will reset the device to factory default settings.

When the zero button is disabled (when this setting is set to 1), inputs to the button will be completely ignored -- 
factory resets and zeroing must be performed over CAN or other supported message layers.

.. _setting_velocity_window:

Controls the number of samples used to average in the velocity window. Samples occur once every 250 microseconds.