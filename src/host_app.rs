use std::path::PathBuf;
use std::sync::Arc;

use gpui::{Image, ImageFormat};

use crate::Error;
use crate::platform;

/// The application the user drags into System Settings.
#[derive(Clone)]
pub struct HostApp {
    /// Name shown in the drag row.
    pub display_name: String,
    /// The `.app` bundle, or the executable when the process runs unbundled.
    pub path: PathBuf,
    /// Icon shown in the drag row; the row shows an empty frame without one.
    pub icon: Option<Arc<Image>>,
}

impl HostApp {
    /// Describes the running application: `CFBundleDisplayName`, then `CFBundleName`, then the
    /// bundle or executable file stem, with its Finder icon.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Unsupported`] off macOS, [`Error::Io`] when the executable path cannot
    /// be read, and [`Error::HostApp`] when no name can be derived.
    pub fn current() -> Result<Self, Error> {
        let info = platform::host_app_info()?;
        Ok(Self {
            display_name: info.display_name,
            path: info.path,
            icon: info
                .icon_png
                .map(|png| Arc::new(Image::from_bytes(ImageFormat::Png, png))),
        })
    }
}
