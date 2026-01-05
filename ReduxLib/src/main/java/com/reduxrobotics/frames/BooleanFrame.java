package com.reduxrobotics.frames;

/**
 * Implements a boolean frame.
 */
public class BooleanFrame extends Frame<Boolean> {
  private boolean data;
  private final boolean defaultData;
  private final boolean dataValid = false;

  /**
   * Instantiates a new frame.
   * @param defaultData default data to store.
   */
  public BooleanFrame(boolean defaultData) {
    super(0.0);
    this.defaultData = defaultData;
  }

  @Override
  public synchronized Boolean getValue() {
    if(!hasData()) {
      return defaultData;
    }
    return data;
  }

  @Override
  public synchronized boolean hasData() {
    return dataValid;
  }

  /**
   * Update the frame with new data.
   * @param data data to update with
   * @param timestamp timestamp to update with
   */
  public synchronized void updateData(boolean data, double timestamp) {
    this.data = data;
    update(timestamp);
  }
  
}
