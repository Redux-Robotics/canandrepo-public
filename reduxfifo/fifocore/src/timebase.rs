use std::sync::LazyLock;

static PROGRAM_EPOCH: LazyLock<(i64, i64)> = LazyLock::new(|| {
    (
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros()
            .max(0) as i64,
        monotonic_us(),
    )
});

/// Yields a monotonic timestamp with a known epoch of January 1st, 1970.
///
/// This is consistent with wpilib's convention.
pub fn now_us() -> i64 {
    if cfg!(feature = "systemcore") {
        monotonic_us()
    } else {
        retimestamp(monotonic_us())
    }
}

/// Monotonic microsecond timer with a platform-defined time base.
#[cfg(unix)]
pub fn monotonic_us() -> i64 {
    // Get the current monotonic time
    let mut mono_time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    unsafe {
        if cfg!(target_os = "macos") {
            libc::clock_gettime(libc::CLOCK_MONOTONIC_RAW, &mut mono_time);
        } else {
            libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut mono_time);
        }
    }
    // Convert the current time from the cpu monotonic clock from seconds + nanoseconds to microseconds
    mono_time.tv_sec as i64 * 1_000_000 + (mono_time.tv_nsec as i64 / 1000_i64)
}

/// Monotonic microsecond timer with a platform-defined time base.
#[cfg(windows)]
pub fn monotonic_us() -> i64 {
    // look, i really didn't want to pull in the windows crate for TWO FUNCTIONS
    #[link(name = "user32")]
    unsafe extern "system" {
        unsafe fn QueryPerformanceFrequency(frequency: *mut i64) -> i32;
        unsafe fn QueryPerformanceCounter(count: *mut i64) -> i32;
    }
    static PERFORMANCE_PERIOD_US: LazyLock<f64> = LazyLock::new(|| {
        let mut f: i64 = 0;
        unsafe {
            QueryPerformanceFrequency(&mut f);
        }
        1_000_000.0 / f as f64
    });

    let mut count: i64 = 0;
    unsafe {
        QueryPerformanceCounter(&mut count);
    }
    ((count as f64) * *PERFORMANCE_PERIOD_US) as i64
}

/// wpilib in sim uses the system time as an epoch.
pub(crate) fn init_program_epoch() {
    LazyLock::force(&PROGRAM_EPOCH);
}

/// Converts an external monotonic microsecond timestamp to our timebase.
///
/// This is mostly relevant in Windows sim, which uses `QueryPerformanceCounter`.
pub fn retimestamp(monotonic_us: i64) -> i64 {
    if cfg!(feature = "systemcore") {
        monotonic_us
    } else {
        let system_offset = PROGRAM_EPOCH.0;
        let monotonic_epoch = PROGRAM_EPOCH.1;
        system_offset
            .wrapping_add(monotonic_us)
            .wrapping_sub(monotonic_epoch)
    }
}
