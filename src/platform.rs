//! Native window placement, System Settings tracking, and host-app lookup.
//!
//! GPUI positions windows relative to a display and offers no way to move a window after it
//! opens, so the overlay is placed through `AppKit` on the `NSWindow` behind the GPUI window.

use std::path::PathBuf;

use crate::motion::ScreenRect;

/// The System Settings window the overlay anchors to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SettingsWindow {
    /// Frame of the System Settings window.
    pub frame: ScreenRect,
    /// Visible frame of the screen that shows most of that window.
    pub visible: ScreenRect,
}

/// What the platform knows about the running application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostAppInfo {
    /// Name to show in the drag row.
    pub display_name: String,
    /// The `.app` bundle, or the executable when the process runs unbundled.
    pub path: PathBuf,
    /// The Finder icon for `path`, PNG-encoded.
    pub icon_png: Option<Vec<u8>>,
}

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(not(target_os = "macos"))]
mod unsupported;
#[cfg(not(target_os = "macos"))]
pub use unsupported::*;
