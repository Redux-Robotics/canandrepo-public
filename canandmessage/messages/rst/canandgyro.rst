.. _msg_yaw_output:

Periodic frame with the Canandgyro's yaw value at full 32-bit float precision in radians.

.. _msg_angular_position_output:

Periodic frame with an angular position quaternion.

The quaternion is normalized and all values are scaled between -1.0 to 1.0 inclusive with one LSB being ``1/32767``-th of a magnitude.
Users are still encouraged to re-normalize this quaternion if converted to other representations.

The minimum step size of this output when converted to Euler angles is approximately 0.004 degrees.

.. _msg_angular_velocity_output:

Periodic frame containing angular velocity. 

.. _msg_acceleration_output:
Periodic frame containing linear acceleration.

.. _msg_calibrate:

Manually triggers gyro calibration. This takes about 5 seconds.

.. _msg_calibration_status:

Sent from the device when calibration completes.

.. _msg_status:

Status frame containing active and sticky fault data as well as temperature. 

.. _setting_yaw_frame_period:

.. _setting_angular_position_frame_period:

.. _setting_angular_velocity_frame_period:

.. _setting_acceleration_frame_period:

.. _setting_set_yaw:

Updates the yaw (Z-axis) rotation with the new sent angular position.

In the firmware, this yaw is achieved by solving for the transformation required to rotate the currently
integrated angular position quaternion to the new yaw angle. 

For example, where the current quaternion has components :math:`w + x \hat{\mathbf{i}} + y\hat{\mathbf{j}} + z\hat{\mathbf{k}}` with 
existing yaw :math:`\phi` and new to-be-set yaw :math:`\phi_n` the new quaternion is calculated by the following:

.. math::

    \left[\cos\left(\frac{1}{2}(\phi_n - \phi)\right) + \sin\left(\frac{1}{2}(\phi_n - \phi)\right)\hat{\mathbf{k}}\right] \cdot
    \left(w + x \hat{\mathbf{i}} + y\hat{\mathbf{j}} + z\hat{\mathbf{k}}\right)

.. _setting_set_pose_positive_w:

Updates the current pose angular position quaternion, assuming a positive W value.

As settings are only 48 bits wide, only three components of a full normalized quaternion are sent, 
with the W (real) component being calculated as so (after re-scaling to -1.0 to 1.0 from the integer encoding):

.. math::
    w = \sqrt{1.0 - x^2 + y^2 + z^2}

If :math:`x^2 + y^2 + z^2 > 1.0`, the W component is assumed to be 0 
and X, Y, and Z are rescaled such the quaternion still has a norm of 1.

.. _setting_set_pose_negative_w:

Updates the current pose angular position quaternion, assuming a negative W value.

As settings are only 48 bits wide, only three components of a full normalized quaternion are sent, 
with the W (real) component being calculated as so (after re-scaling to -1.0 to 1.0 from the integer encoding):

.. math::
    w = -\sqrt{1.0 - x^2 + y^2 + z^2}

If :math:`x^2 + y^2 + z^2 > 1.0`, the W component is assumed to be 0 
and X, Y, and Z are rescaled such the quaternion still has a norm of 1.

.. _setting_gyro_x_sensitivity:

Read-only factory-calibrated X-axis sensitivity value

.. _setting_gyro_y_sensitivity:

Read-only factory-calibrated Y-axis sensitivity value

.. _setting_gyro_z_sensitivity:

Read-only factory-calibrated Z-axis sensitivity value

.. _setting_x_zro_offset:

Saved X-axis angular zero-motion velocity offset. 

This is used during the power-on calibration to seed the initial offset vector.

If :ref:`calibration<msg_calibrate>` is run with the :ref:`SAVE_ZRO<enum_calibration_type>` calibration type,
this setting will be updated with the new calculated offset at the end of calibration.

.. _setting_y_zro_offset:

Saved Y-axis zero-motion zero motion velocity offset. 

This is used during the power-on calibration to seed the initial offset vector.

If :ref:`calibration<msg_calibrate>` is run with the :ref:`SAVE_ZRO<enum_calibration_type>` calibration type,
this setting will be updated with the new calculated offset at the end of calibration.

.. _setting_z_zro_offset:

Saved Z-axis zero-motion angular velocity offset. 

This is used during the power-on calibration to seed the initial offset vector.

If :ref:`calibration<msg_calibrate>` is run with the :ref:`SAVE_ZRO<enum_calibration_type>` calibration type,
this setting will be updated with the new calculated offset at the end of calibration.

.. _setting_gyro_zro_offset_temperature:

Saved temperature associated with the saved x/y/z-axis zero-motion angular velocity offset.

If :ref:`calibration<msg_calibrate>` is run with the :ref:`SAVE_ZRO<enum_calibration_type>` calibration type,
this setting will be updated with the new temperature at the end of calibration.

.. _setting_temperature_calibration_x_0:

Reserved.

.. _setting_temperature_calibration_y_0:

Reserved.

.. _setting_temperature_calibration_z_0:

Reserved.

.. _setting_temperature_calibration_t_0:

Reserved.

.. _setting_temperature_calibration_x_1:

Reserved.

.. _setting_temperature_calibration_y_1:

Reserved.

.. _setting_temperature_calibration_z_1:

Reserved.

.. _setting_temperature_calibration_t_1:

Reserved.