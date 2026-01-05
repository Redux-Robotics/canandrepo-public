// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.canand;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.ArrayList;
import java.util.List;

import edu.wpi.first.hal.ThreadsJNI;
import edu.wpi.first.wpilibj.DriverStation;
import edu.wpi.first.wpilibj.Notifier;

/**
 * Class that runs the CAN packet ingest loop, and starts the CANLink interface.
 * 
 * <p>In order to start the Redux CANLink server, either instantiate any Redux device
 * in your robot code or call</p>
 * 
 * <pre>
 * CanandEventLoop.getInstance();
 * </pre>
 * somewhere in an init function
 * 
 */
public class CanandEventLoop implements Runnable {
    private static CanandEventLoop instance = null;
    private static Thread runner = null;
    private static boolean shouldRun = false;
    private static Notifier deviceChecker;

    private List<DeviceEntry> listeners;
    //private HashMap<CanandDevice, CheckState> checkedDevices;
    private boolean enableDevicePresenceWarnings = true;

    private static enum CheckState {
        kUnchecked,
        kDoNotCheck,
        kWaitingOnFirmwareVersion,
        kConnected,
        kDisconnected,
    }

    private static class DeviceEntry {
        CanandDevice device;
        CheckState state = CheckState.kUnchecked;
        double presenceThreshold = 2.0;
        int repeatTimeout = 20;
        int firmwareCheckAttempts = 5;
        public DeviceEntry(CanandDevice device) {
            this.device = device;
        }
    }

    private CanandEventLoop() {
        listeners = new ArrayList<>();
        ReduxJNI.init();
        deviceChecker = new Notifier(this::deviceCheckerTask);
        deviceChecker.startPeriodic(0.5);
    } 

    private DeviceEntry getDeviceEntryIfExists(CanandDevice dev) {
        // yes this is O(n) but if you have more than 100 device objects you are probably doing
        // something wrong.
        for (DeviceEntry ent: listeners) {
            if (ent.device == dev) return ent;
        }
        return null;
    }


    /**
     * Returns a handle to the event loop. Starts it if not started.
     * @return the singleton instance
     */
    public synchronized static CanandEventLoop getInstance() {
        if (instance == null) instance = new CanandEventLoop();
        if (runner == null) {
            shouldRun = true;
            runner = new Thread(instance);
            runner.start();
        }

        return instance;
    }

    /**
     * Set whether to enable device presence warnings to the driver station globally 
     * (defaults to true).
     * 
     * @param enabled true to enable, false to suppress
     */
    public synchronized void setGlobalDevicePresenceWarnings(boolean enabled) {
        enableDevicePresenceWarnings = enabled;
    }

    /**
     * Set whether to enable device presence warnings to the driver station for a single device 
     * (defaults to true).
     * 
     * @param device the CanandDevice to consider
     * @param enabled true to enable, false to suppress
     */
    public synchronized void setDevicePresenceWarnings(CanandDevice device, boolean enabled) {
        DeviceEntry ent = getDeviceEntryIfExists(device);
        if (ent == null) return;
        if (enabled) {
            ent.state = device.isConnected(ent.presenceThreshold) ? CheckState.kConnected: CheckState.kDisconnected;
        } else {
            ent.state = CheckState.kDoNotCheck;
        }
    }

    /**
     * Sets the device presence threshold of how many seconds must pass without a message for the
     * device checker to consider them disconnected from bus.
     * 
     * @param device the CanandDevice to act on
     * @param threshold the new threshold, in seconds. 
     */
    public synchronized void setDevicePresenceThreshold(CanandDevice device, double threshold) {
        DeviceEntry ent = getDeviceEntryIfExists(device);
        if (ent == null) return;
        ent.presenceThreshold = threshold;
    }

    private void reportMissingDevice(String deviceName) {
        DriverStation.reportError(
            String.format("Not receiving data from %s - likely disconnected from robot. Check wiring and/or frame periods!",
            deviceName), false);
    }

    /**
     * Periodic function that checks to ensure devices actually exist on bus.
     */
    private synchronized void deviceCheckerTask() {
        if (CanandUtils.getFPGATimestamp() < 2.0) { return; }
        for (DeviceEntry ent: listeners) {
            CanandDevice device = ent.device;
            if (device.getAddress() == null) continue; // object not done constructing
            switch (ent.state) {
                case kUnchecked: {
                    if (device.getMinimumFirmwareVersion() == null) {
                        // skip this check entirely
                        ent.state = CheckState.kDisconnected;
                        break;
                    }
                    device.sendCANMessage(CanandDeviceDetails.Msg.kSettingCommand,
                        CanandDeviceDetails.Msg.constructSettingCommand(
                            CanandDeviceDetails.Enums.SettingCommand.kFetchSettingValue, 
                            CanandDeviceDetails.Stg.kFirmwareVersion
                        ), 2);
                    ent.state = CheckState.kWaitingOnFirmwareVersion;
                    break;
                }
                case kWaitingOnFirmwareVersion: {
                    if (device.getFirmwareVersion() == null && ent.firmwareCheckAttempts > 0) {
                        device.sendCANMessage(CanandDeviceDetails.Msg.kSettingCommand,
                            CanandDeviceDetails.Msg.constructSettingCommand(
                                CanandDeviceDetails.Enums.SettingCommand.kFetchSettingValue, 
                                CanandDeviceDetails.Stg.kFirmwareVersion
                            ), 2);
                        ent.firmwareCheckAttempts -= 1;
                    } else {
                        // Presumably after the next wakeup we should have received a firmware verison in response
                        device.checkReceivedFirmwareVersion();
                        ent.state = device.isConnected(ent.presenceThreshold) ? CheckState.kConnected : CheckState.kDisconnected;
                    }
                    break;
                }
                case kConnected: {
                    if (!device.isConnected(ent.presenceThreshold) && enableDevicePresenceWarnings) {
                        reportMissingDevice(device.toString());
                        ent.state = CheckState.kDisconnected;
                    }
                    break;
                }
                case kDisconnected: {
                    if (device.isConnected(ent.presenceThreshold)) {
                        ent.state = CheckState.kConnected;
                        ent.repeatTimeout = 20;
                    } else if (ent.repeatTimeout-- <= 0) {
                        reportMissingDevice(device.toString());
                        ent.repeatTimeout = 20;
                    }
                    break;
                }
                case kDoNotCheck: {
                    break;
                }
            }

        }
    }

    /**
     * Adds a listener for CAN messages to the event loop.
     * @param listener a {@link CanandDevice}
     */
    public synchronized void addListener(CanandDevice listener) {
        listeners.add(new DeviceEntry(listener));
    }

    /**
     * Removes a listener for CAN messages, if it is in fact listening.
     * @param listener the {@link CanandDevice} to remove
     */
    public synchronized void removeListener(CanandDevice listener) {
        DeviceEntry entToRemove = null;
        for (DeviceEntry ent : listeners) {
            if (ent.device == listener) {
                entToRemove = ent;
                break;
            }
        }
        if (entToRemove != null) listeners.remove(entToRemove);
    }

    public void run() {
        ThreadsJNI.setCurrentThreadPriority(true, 30);
        System.out.println("[ReduxLib] CanandEventLoop started.");
        CanandMessage msg = new CanandMessage();
        int bufsz = 32;

        ByteBuffer bb = ReduxJNI.allocateMessageBuffer(bufsz);
        while (shouldRun) {
            bb.clear();
            int status_count = ReduxJNI.batchWaitForCANMessage(bb, bufsz);
            if (status_count == -1) break; // the can queue has been interrupted, so we should exit
            else if (status_count <= 0) continue;
            bb.order(ByteOrder.LITTLE_ENDIAN);
            // find way to shut down cleanly?
            synchronized (this) {
                for (int i = 0; i < Math.min(status_count, bufsz); i++) {
                    msg.updateFromByteBuf(bb);
                    for (DeviceEntry ent: listeners) {
                        try {
                            CanandDevice listener = ent.device;
                            CanandAddress addr = listener.getAddress();
                            if (addr == null) continue;
                            if (addr.msgMatches(msg)) {
                                listener.preHandleMessage(msg);
                                listener.handleMessage(msg);
                            }
                        } catch (Exception e) {
                            DriverStation.reportError("Exception in CanandEventLoop message listener: \n" 
                            + e.getClass().getName() + ": " + e.getMessage(), e.getStackTrace());
                        }
                    }
                }
            }
        }
        ReduxJNI.deallocateBuffer(bb);
        System.out.println("[ReduxLib] CanandEventLoop exit.");

    }
}
