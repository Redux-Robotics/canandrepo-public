// Copyright (c) Bagholders of Redux Robotics and other contributors.
// This is open source and can be modified and shared under the Mozilla Public License v2.0.

#pragma once
#include <units/time.h>
#include <optional>
#include <mutex>
#include <condition_variable>
#include <tuple>
#include <unordered_map>
#include <set>
#include <chrono>
#include <algorithm>
#include <functional>
namespace redux::frames {

/**
 * Immutable container class for timestamped values.
*/
template <typename T>
class FrameData {
  public:
    FrameData() = default;
    /**
     * Constructs a new FrameData object.
     * 
     * @param value The value to hold.
     * @param timestamp The timestamp at which the value was received in seconds.
     */
    FrameData(T value, units::second_t timestamp): value{value}, ts{timestamp} {};
    /**
     * Returns the value of the data frame.
     * @return the value the data frame holds.
     */
    inline T GetValue() { return *value; }

    /**
     * Gets the timestamp in seconds of when this value was updated.
     * The time base is relative to the FPGA timestamp.
     * @return the timestamp in seconds.
     */
    inline units::second_t GetTimestamp() { return ts; }
  private:
    std::optional<T> value; // value
    units::second_t ts; // timestamp
};

/**
 * Internal class.
*/
template<typename T>
class FrameListener {
  public:
    /**
     * Instantiates a FrameListener, used internally for WaitForFrames
     * @param cv the condition variable to flag when data is received
     * @param dataLock the associated data lock for the frame listener
    */
    FrameListener(std::condition_variable& cv, std::mutex& dataLock): cv{cv}, dataLock{dataLock}{};
    /**
     * updates the framelistener with the value/timestamp pair, notifying its associated condition variable
     * @param value value to update with
     * @param timestamp timestamp to update with
    */
    void UpdateValue(T value, units::second_t timestamp) {
        std::unique_lock<std::mutex> lock(dataLock);
        this->data = std::optional<FrameData<T>>{FrameData{value, timestamp}};
        this->cv.notify_all(); 
    };
    /** condition variable reference */
    std::condition_variable& cv;
    /** data lock reference */
    std::mutex& dataLock;
    /** payload to hold received data -- check if nullopt for populated-ness */
    std::optional<FrameData<T>> data{std::nullopt};
};

/**
 * Class representing periodic timestamped data received from CAN or other sources.
 * 
 * <p>
 * For applications like latency compensation, we often need both sensor/device data and a timestamp of when the data was received. 
 * Frames provide this by holding the most recently received raw data and the timestamps they were received at and allowing retrieval of both data and timestamps
 * in one FrameData object via Frame.GetFrameData(), avoiding race conditions involving reading data and timestamp separately.
 * Additionally, they allow for synchronous reads through WaitForFrames by notifying when new data has been received. 
 * </p>
 */
template <typename T>
class Frame {
  public:
    /**
     * Constructs a new Frame object.
     * 
     * @param value The initial value to hold.
     * @param timestamp The initial timestamp at which the value was received in seconds.
     */
    Frame(T value, units::second_t timestamp): value{value}, ts{timestamp} {};
    /**
     * Updates the Frame's value, notifying any listeners of new data.
     * 
     * @param value the new value 
     * @param timestamp the new timestamp of the received data
     */
    void Update(T value, units::second_t timestamp) {
        std::unique_lock<std::mutex> lock(frameLock);
        this->value = value;
        this->ts = timestamp;
        for (FrameListener<T>* fl : listeners) {
            fl->UpdateValue(value, ts);
        }
        for (const auto& [key, cb] : callbacks) {
          cb(FrameData<T>{value, ts});
        }
    };
    /**
     * Fetches an immutable FrameData snapshot of the currently stored values
     * @return FrameData of value/timestamp pair
    */
    inline FrameData<T> GetFrameData() { 
      std::unique_lock<std::mutex> lock(frameLock);
      return FrameData<T>{value, ts}; 
    }
    /**
     * Returns the current frame's value.
     * @return the value the data frame holds.
     */
    inline T GetValue() { 
      std::unique_lock<std::mutex> lock(frameLock);
      return value; 
    }

    /**
     * Gets the timestamp in seconds of when this frame was last updated.
     * @return the timestamp in seconds.
     */
    inline units::second_t GetTimestamp() { 
      std::unique_lock<std::mutex> lock(frameLock);
      return ts; 
    }

    /**
     * Add a callback that will be run whenever this Frame gets updated.
     * Example application (may not be applicable)
     * ```cpp
     * // Log Canandmag position FrameData.
     * std::vector<FrameData<units::turn_t>> position_packets;
     * redux::sensors::canandmag::Canandmag enc0{0};
     * 
     * enc0.GetPositionFrame().AddCallback([&](FrameData<units::turn_t> frameData) {
     *     position_packets.push_back(frameData);
     * });
     * // Timestamped data is now appended to the Vector.
     * 
     * ```
     * @param callback the callback
     * @return an handle key that can be used to unregister the callback later 
     */
    inline uint32_t AddCallback(std::function<void(FrameData<T>)> callback) {
      callbacks[key++] = callback;
      return key;
    }

    /**
     * Unregister a callback run whenever this Frame gets updated.
     * @param key the key returned by AddCallback
     * @return true on unregister, false if the callback didn't exist
     */
    inline bool RemoveCallback(uint32_t key) {
      return callbacks.erase(key);
    }

    /**
     * Internal use function (for WaitForFrames)
     * @param listener listener pointer to add
    */
    inline void AddListener(FrameListener<T>* listener) { 
      std::unique_lock<std::mutex> lock(frameLock);
      listeners.insert(listener); 
    }

    /**
     * Internal use function (for WaitForFrames)
     * @param listener listener pointer to remove. 
     * you must remove your listener before the end of its life or you will cause memory corruption 
    */
    inline void RemoveListener(FrameListener<T>* listener) { 
      std::unique_lock<std::mutex> lock(frameLock);
      listeners.erase(listener); 
    }


  private:
    T value; // value
    units::second_t ts; // timestamp
    std::mutex frameLock;
    std::set<FrameListener<T>*> listeners;
    std::unordered_map<uint32_t, std::function<void(FrameData<T>)>> callbacks;
    uint32_t key{0};
};

/**
 * Waits for all Frames to have transmitted a value. 
 * Either returns an std::tuple of FrameData\<T\>; representing the data from corresponding frames passed in (in the order they are passed in) or std::nullopt if timeout or interrupt is hit.
 * 
 * Code example:
 * ```cpp
 *  // Keep in mind this code sample will likely cause timing overruns if on the main thread of your robot code.
 *  // Device definitions:
 *  redux::sensors::canandmag::Canandmag enc0{0};
 *  redux::sensors::canandmag::Canandmag enc1{1};
 * 
 *  // wait up to 40 ms for position and velocity data to come in from two Canandmags
 *  auto data = redux::frames::WaitForFrames(40_ms, enc0.GetPositionFrame(), enc0.GetVelocityFrame(), enc1.GetPositionFrame());
 *  if (!data.has_value()) {
 *    fmt::print("WaitForFrames timed out before receiving all data\n");
 *  } else {
 *    redux::frames::FrameData<units::turn_t> posFrame;
 *    redux::frames::FrameData<units::turn_t> posFram1;
 *    redux::frames::FrameData<units::turns_per_second_t> velFrame;
 *
 *    // populates the above FrameData variables with the received data (unpacks the tuple)
 *    std::tie(posFrame, velFrame, posFram1) = *data;
 *
 *    // fetches the maximum timestamp across all received timestamps (the "latest" value)
 *    units::second_t maxTs = redux::frames::MaxTimestamp(*data);
 *
 *    // prints the received frame value and how far behind the latest received CAN timestamp it was
 *    fmt::print("posFrame: {}, {}\n", posFrame.GetValue(), 
 *               (units::millisecond_t) (maxTs - posFrame.GetTimestamp()));
 *    fmt::print("velFrame: {}, {}\n", velFrame.GetValue(), 
 *               (units::millisecond_t) (maxTs - velFrame.GetTimestamp()));
 *    fmt::print("posFram1: {}, {}\n", posFram1.GetValue(), 
 *               (units::millisecond_t) (maxTs - posFram1.GetTimestamp()));
 *
 *  } 
 * ```
 * 
 * @param timeout maximum seconds to wait for before giving up
 * @param frames references to Frames to wait on. Position in argument list corresponds to position in the returned FrameData tuple.
 * @return a tuple of FrameData\<T\> representing the data from corresponding frames passed in or null if timeout or interrupt is hit.
 */
template<typename...T>
std::optional<std::tuple<FrameData<T>...>> WaitForFrames(units::second_t timeout, Frame<T>&... frames) {
    constexpr auto sec = std::chrono::seconds(1);
    std::condition_variable cv; 
    std::mutex dataLock;

    auto listeners = std::make_tuple(std::make_pair(FrameListener<T>(cv, dataLock), &frames)...);
    {
        std::unique_lock<std::mutex> lock(dataLock);
        std::apply([](std::pair<FrameListener<T>, Frame<T>*>&... i) {(i.second->AddListener(&i.first), ...);}, listeners);

        if (!cv.wait_for(lock, timeout.to<double>() * sec, [&]{
            return std::apply([](std::pair<FrameListener<T>, Frame<T>*>&... i) { return ((i.first.data != std::nullopt) && ...); }, listeners);
        })) {
            // timeout
            // perform cleanup and return std::nullopt
            std::apply([](std::pair<FrameListener<T>, Frame<T>*>&... i) {(i.second->RemoveListener(&i.first), ...);}, listeners);
            return std::nullopt;
        }
        std::apply([](std::pair<FrameListener<T>, Frame<T>*>&... i) {(i.second->RemoveListener(&i.first), ...);}, listeners);
    }
    return std::apply([](std::pair<FrameListener<T>, Frame<T>*>&... i) { return std::make_tuple(*(i.first.data)...); }, listeners);
}


/**
 * Returns the max timestamp from a tuple of FrameData objects.
 * Most useful for getting the "latest" CAN timestamp from a result of WaitForFrames.
 * @param frameData value from WaitForFrames
 * @return the maximum timestamp
*/
template<typename...T>
units::second_t MaxTimestamp(std::tuple<FrameData<T>...> frameData) {
  return std::apply([](FrameData<T>&... i) { 
    // we don't have std::max( std::initalizer_list<T> ilist) for some reason so we have to do this
    auto ilist = {(i.GetTimestamp())...}; 
    return *std::max_element(ilist.begin(), ilist.end());
  }, frameData);
}

}