/// The current monotonic time.
/// This is the FPGA time if wpihal support is compiled in, otherwise just [`monotonic_us`]
#[cfg(feature = "wpihal-rio")]
pub fn now_us() -> i64 {
    wpihal_rio::get_fpga_time().unwrap_or(0) as i64
}

/// The current monotonic time.
/// This is the FPGA time if wpihal support is compiled in, otherwise just [`monotonic_us`]
#[cfg(feature = "wpihal-mrc")]
pub fn now_us() -> i64 {
    wpihal_mrc::get_fpga_time().unwrap_or(0) as i64
}

/// The current monotonic time.
/// This is the FPGA time if wpihal support is compiled in, otherwise just [`monotonic_us`]
#[cfg(not(any(feature = "wpihal-rio", feature = "wpihal-mrc")))]
pub fn now_us() -> i64 {
    monotonic_us()
}

#[allow(unused)]
pub fn retimestamp(ts_us: i64, other_time_us: i64) -> u64 {
    // Get the current monotonic time
    // Get the current fpga time
    let fpga_time_us = now_us();

    // Convert the current time from the other timestamp
    let offset_us = fpga_time_us - other_time_us;
    (ts_us + offset_us) as u64
}

#[allow(unused)]
pub fn retimestamp_from_monotonic(ts_us: i64) -> u64 {
    #[cfg(feature = "wpihal-rio")]
    {
        retimestamp(ts_us, monotonic_us())
    }
    #[cfg(not(feature = "wpihal-rio"))]
    {
        ts_us as u64
    }
}

/// Yields a monotonic timebase, decoupled from wpilib.
///
/// This is `CLOCK_MONOTONIC` on unix and `QueryPerformanceCounter` on windows.
#[cfg(unix)]
pub fn monotonic_us() -> i64 {
    // Get the current monotonic time
    let mut mono_time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    unsafe {
        libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut mono_time);
    }
    // Convert the current time from the cpu monotonic clock from seconds + nanoseconds to microseconds
    mono_time.tv_sec as i64 * 1_000_000 + (mono_time.tv_nsec as i64 / 1000_i64)
}

// no idea if this works but i really didn't want to pull in the windows crate

#[cfg(windows)]
#[link(name = "user32")]
unsafe extern "system" {
    unsafe fn QueryPerformanceFrequency(frequency: *mut i64) -> i32;
    unsafe fn QueryPerformanceCounter(count: *mut i64) -> i32;
}

#[cfg(windows)]
static PERFORMANCE_FREQUENCY: std::sync::LazyLock<i64> = std::sync::LazyLock::new(|| {
    let mut f: i64 = 0;
    unsafe {
        QueryPerformanceFrequency(&mut f);
    }
    f
});

/// Yields a monotonic timebase, decoupled from wpilib.
///
/// This is `CLOCK_MONOTONIC` on unix and `QueryPerformanceCounter` on windows.
#[cfg(windows)]
pub fn monotonic_us() -> i64 {
    let mut count: i64 = 0;
    unsafe {
        QueryPerformanceCounter(&mut count);
    }
    ((count as f64) / *PERFORMANCE_FREQUENCY as f64 * 1_000_000.0) as i64
}
