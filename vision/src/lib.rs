pub mod monitor;
pub use monitor::{MonitorData, SafeMonitor};

pub mod capture_screenshot_by_window;
pub use capture_screenshot_by_window::{
    capture_all_visible_windows, CapturedWindow, WindowFilters,
};
