.. _msg_can_id_arbitrate:

Selects the conflicting device to use during a CAN id conflict.
All other conflicting devices will cease transmitting packets until they are also explicitly enabled via this packet.

May have unintended consequences on non-CAN transport mediums.

.. _msg_can_id_error:

Broadcasted on repeat when a CAN id conflict is detected until an arbitrate packet is received. All other CAN transmit is disabled.

.. _msg_can_id_error:

Sent to device to execute a command on the settings subsystem. Packet length is variable depending on command but is at least 1 byte.

.. _msg_setting_command:

Sent to the device to operate on the settings subsystem. 

Devices may add their own setting commands but they will typically at least have:

* fetch all settings (id 0x0)
* reset all applicable settings to factory default (id 0x1)
* fetch setting value (id 0x2)

Most setting commands (e.g. reset to factory default and get all settings) are allowed to have a data length code of 1, however,
the fetch setting value command requires a data length of at least 2 bytes to also specify the setting index to fetch.

The most typical use case is likely to fetch a specific setting value; to fetch firmware version for example one might send this packet
with the payload ``{0x2, 0x6}`` and wait for a :ref:`report setting<msg_report_setting>` to report the setting.

.. _msg_set_setting:

Sent to device to change a setting by address.

If the setting exists, a :ref:`report setting<msg_report_setting>` packet will be sent in reply, with the data of the setting echoed back
and information on whether or not the setting set succeeded.

.. _msg_report_setting:

Sent to report a setting value from the device.

These messages can be triggered by:

* a setting change via :ref:`the set setting message<msg_set_setting>`
* the fetch setting value :ref:`setting command<enum_setting_command>`
* a reset to factory default :ref:`setting command<enum_setting_command>`
* other device-specific mechanisms including device-specific setting commands

The setting flags include information on whether or not the setting set was successful as well as the setting data that was sent.

Sent after a setting change via or on the fetch settings and factory reset 
Setting changes (as of v2024) will always include the "settings flag" field.

.. _msg_clear_sticky_faults:

Sent to device to clear all sticky faults (sets the sticky faults to 0 until faults become active again)

.. _msg_status:

Periodic frame containing status information about the device.

Typically this contains active and sticky faults as well as environmental information such as device temperature.
The actual composition of this message will vary from device to device, but it is guarenteed to be 8 bytes long. 
Consult individual device documentation pages for more information.

This frame cannot be disabled.

.. _msg_party_mode:

Configures party mode to the device.

Non-zero values will prompt the onboard RGB LED of the device to cycle various colors to help identify 
where it physically sits on a robot.

A zero value stops the cycling.

.. _msg_ota_data:

Data container packet used during the OTA process. For more specifics, consult the rdxota crate.

.. _msg_ota_to_host:

OTA control messages sent from device to OTA host. For more specifics, consult the rdxota crate.

.. _msg_ota_to_device:

OTA control messages sent from OTA host to device. For more specifics, consult the rdxota crate.

.. _msg_enumerate:

Sent by the device upon an enumerate request, or every 100 milliseconds if the device is stuck in OTA bootloader.

The exact format of enumerate request may vary between communication mediums:

* for a CAN bus an enumerate request is a message with an extended (29-bit) arbitration ID of 0xE0000 

.. _setting_can_id:

Sets the 6-bit device id, ranging from 0 to 63.
This allows multiple of a device to share a bus.

Defaults to 0 but does not reset on factory resets.

.. _setting_name_0:

First 6 bytes of the device name.

Having a null byte will terminate the name field at that byte.

.. _setting_name_1:

Middle 6 bytes of the device name.

Having a null byte will terminate the name field at that byte.

.. _setting_name_2:

Last 6 bytes of the name field.

Having a null byte will terminate the name field at that byte.

All 6 bytes can be non-null and the name will be 18 characters long.

.. _setting_status_frame_period:

Period between the transmission of :ref:`status frame messages<msg_status>` in milliseconds.
This frame cannot be disabled (as Alchemist uses it to detect devices).

.. _setting_serial_number:

Read-only setting of the device's serial number.

.. _setting_firmware_version:

Read-only setting value of the device's firmware version.

.. _setting_chicken_bits:

Chicken bits that vary in meaning from device to device used for debugging/side band purposes.

**Not intended for end-user use!** Fiddling around with this register may reduce device performance,
cause hardware faults, or otherwise make things explode. Do not touch this register unless directed to by support.

.. _setting_device_type:

Read-only device type identifier.

If multiple types of device share the same FRC-CAN device class, e.g. 
two device variants that do similar things but may or may not have the same message API, 
this setting can be used to disambiguate between them.