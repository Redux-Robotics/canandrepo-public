// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

package com.reduxrobotics.sensors.canandcolor;

/**
 * Enum representing the integration period of the Canandcolor's proximity sensor IC.
 * 
 * <p>This determines how long the proximity sensor spends collecting reflected IR light to determine a proximity reading,
 * which also determines the sample rate. 
 * Longer integration periods increase sensitivity but shorter integration periods have faster sample rates.</p>
 * 
 */
public enum ProximityPeriod {
    /** 5 millisecond update period. */
    k5ms(CanandcolorDetails.Enums.DistanceIntegrationPeriod.kPeriod5Ms),
    /** 10 millisecond update period. */
    k10ms(CanandcolorDetails.Enums.DistanceIntegrationPeriod.kPeriod10Ms),
    /** 20 millisecond update period. */
    k20ms(CanandcolorDetails.Enums.DistanceIntegrationPeriod.kPeriod20Ms),
    /** 40 millisecond update period. */
    k40ms(CanandcolorDetails.Enums.DistanceIntegrationPeriod.kPeriod40Ms);
    private int index;
    ProximityPeriod(int index) { this.index = index; }

    /**
     * Returns a corresponding enum from the index.
     * @param idx the index to fetch.
     * @return a valid enum. If the index value is invalid it will return the default.
     */
    public static ProximityPeriod fromIndex(int idx) { 
        switch (idx) {
            case CanandcolorDetails.Enums.DistanceIntegrationPeriod.kPeriod5Ms: { return ProximityPeriod.k5ms; }
            case CanandcolorDetails.Enums.DistanceIntegrationPeriod.kPeriod10Ms: { return ProximityPeriod.k10ms; }
            case CanandcolorDetails.Enums.DistanceIntegrationPeriod.kPeriod20Ms: { return ProximityPeriod.k20ms; }
            case CanandcolorDetails.Enums.DistanceIntegrationPeriod.kPeriod40Ms: { return ProximityPeriod.k40ms; }
            default: { return ProximityPeriod.k20ms; }
        }
    }

    /**
     * Gets the corresponding index for the value in question.
     * @return the index value for the enum (used internally)
     */
    public int getIndex() { return index; }
}