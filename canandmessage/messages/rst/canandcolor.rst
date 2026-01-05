.. _msg_distance_output:

Periodic frame with the Canandcolor's raw 16-bit distance output.

This distance measurement increases approximately linearly as objects move away from the sensor until sensor reading ability drops off.

.. _msg_color_output:

Periodic frame with the Canandcolor's color sensor output, with up to 20-bit precision.

The red/blue/green channels will be left-shifted such that the largest possible sensable value is always the one closest to 0xfffff. 
For example, if the color sensor is configured in 25 ms/16 bit mode, the output values are left-shifted by 4 bits such that 
0xffff0 is the largest senseable magnitude, rather than 0xffff. 

The color integration period is returned in the leftover 4 bits of the packet.

.. _msg_digital_output:

Periodic frame with the Canandcolor's digital output state, including digout slots for both channels (both overall evaluation and individual slot conditions).
Digout channel slots are always evaluated, even if the physical digital outputs themselves are disabled -- this allows them to be also be used as 
programmable interrupt flags for CAN-only usage.

The sticky flags raise whenever digout1 or digout2 have evaluated to true since the last time they have been cleared. 
This can be used to check if a sensor condition has ever been true between robot loop iterations.

.. _msg_clear_sticky_digout:

Clears the sticky digout state for both digout1 and digout2 associated with the :ref:`digital output<msg_digital_output>` packet.

.. _msg_status:

Periodic frame containing status information about the device.

This contains active and sticky faults as well as temperature, although the temperature reading is best described as a general approximation.

This frame cannot be disabled.

.. _setting_color_extra_frame_mode:

Sets the :ref:`extra frame mode<enum_extra_frame_mode>` for :ref:`color sensor messages.<msg_color_output>` 

.. _setting_distance_extra_frame_mode:

Sets the :ref:`extra frame mode<enum_extra_frame_mode>` for :ref:`distance sensor messages.<msg_distance_output>` 

.. _setting_lamp_brightness:

Sets the LED brightness for the lamp (white) LED.

.. _setting_color_integration_period:

Sets the integration period for the color sensor.

.. _setting_distance_integration_period:

Sets the integration period for the distance sensor.

.. _setting_digout1_output_config:

Sets up how the physical DIG-1 GPIO pin should act.

This pin does not support PWM output.

.. _setting_digout2_output_config:

Sets up how the physical DIG-2 GPIO pin should act.

.. _setting_digout1_message_on_change:

Sets how extra :ref:`digout messages<msg_digital_output>` should be sent when the value of digout channel 1 changes.

.. _setting_digout2_message_on_change:

Sets how extra :ref:`digout messages<msg_digital_output>` should be sent when the value of digout channel 2 changes.

.. _setting_digout1_config_0:

Digout slot channel 1 index 0

.. _setting_digout1_config_1:

Digout slot channel 1 index 1

.. _setting_digout1_config_2:

Digout slot channel 1 index 2

.. _setting_digout1_config_3:

Digout slot channel 1 index 3

.. _setting_digout1_config_4:

Digout slot channel 1 index 4

.. _setting_digout1_config_5:

Digout slot channel 1 index 5

.. _setting_digout1_config_6:

Digout slot channel 1 index 6

.. _setting_digout1_config_7:

Digout slot channel 1 index 7

.. _setting_digout1_config_8:

Digout slot channel 1 index 8

.. _setting_digout1_config_9:

Digout slot channel 1 index 9

.. _setting_digout1_config_10:

Digout slot channel 1 index 10

.. _setting_digout1_config_11:

Digout slot channel 1 index 11

.. _setting_digout1_config_12:

Digout slot channel 1 index 12

.. _setting_digout1_config_13:

Digout slot channel 1 index 13

.. _setting_digout1_config_14:

Digout slot channel 1 index 14

.. _setting_digout1_config_15:

Digout slot channel 1 index 15 

.. _setting_digout2_config_0:

Digout slot channel 2 index 0

.. _setting_digout2_config_1:

Digout slot channel 2 index 1

.. _setting_digout2_config_2:

Digout slot channel 2 index 2

.. _setting_digout2_config_3:

Digout slot channel 2 index 3

.. _setting_digout2_config_4:

Digout slot channel 2 index 4

.. _setting_digout2_config_5:

Digout slot channel 2 index 5

.. _setting_digout2_config_6:

Digout slot channel 2 index 6

.. _setting_digout2_config_7:

Digout slot channel 2 index 7

.. _setting_digout2_config_8:

Digout slot channel 2 index 8

.. _setting_digout2_config_9:

Digout slot channel 2 index 9

.. _setting_digout2_config_10:

Digout slot channel 2 index 10

.. _setting_digout2_config_11:

Digout slot channel 2 index 11

.. _setting_digout2_config_12:

Digout slot channel 2 index 12

.. _setting_digout2_config_13:

Digout slot channel 2 index 13

.. _setting_digout2_config_14:

Digout slot channel 2 index 14

.. _setting_digout2_config_15:

Digout slot channel 2 index 15 