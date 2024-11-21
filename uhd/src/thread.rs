use crate::check_status;
use crate::error::Error;

// These values are not defined in the C API, but they are defined in the C++ API
const DEFAULT_THREAD_PRIORITY: f32 = 0.5;
const DEFAULT_THREAD_REALTIME: bool = true;

pub fn set_thread_priority(priority: Option<f32>, realtime: Option<bool>) -> Result<(), Error> {
    let priority = if let Some(priority) = priority {
        priority
    } else {
        DEFAULT_THREAD_PRIORITY
    };

    let realtime = if let Some(realtime) = realtime {
        realtime
    } else {
        DEFAULT_THREAD_REALTIME
    };

    check_status(unsafe { uhd_sys::uhd_set_thread_priority(priority, realtime) })
}
