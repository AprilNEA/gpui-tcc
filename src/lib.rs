//! Guides macOS users through granting a privacy permission to a GPUI application.
//!
//! Calling [`Guide::present`] opens the matching Privacy & Security pane in System Settings and
//! floats a small panel over it that the user drags into the pane's app list. The panel follows
//! the System Settings window, hides when another application comes to the front, and can fly in
//! from a control in the host window so the user sees where it came from.
//!
//! The crate compiles on every platform GPUI supports; on anything other than macOS,
//! [`Guide::present`] returns [`Error::Unsupported`].
//!
//! ```no_run
//! use gpui::{App, Bounds, Pixels, Window};
//! use gpui_tcc::{Guide, PrivacyPanel};
//!
//! fn request_accessibility(button: Bounds<Pixels>, window: &Window, cx: &mut App) {
//!     Guide::new(PrivacyPanel::Accessibility)
//!         .launch_from(button, window)
//!         .present(cx)
//!         .expect("System Settings guide");
//! }
//! ```
//!
//! A port of [Permiso](https://github.com/zats/permiso) by Sash Zats.

mod assistant;
mod host_app;
mod motion;
mod overlay;
mod panel;
mod platform;

use gpui::{App, Bounds, Pixels, SharedString, Window};

pub use host_app::HostApp;
pub use panel::PrivacyPanel;

use motion::ScreenRect;

/// Why a guide could not be presented.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The current platform has no System Settings privacy panes.
    #[error("System Settings guidance is only available on macOS")]
    Unsupported,
    /// GPUI refused to open the overlay window.
    #[error("could not open the overlay window")]
    OpenWindow(#[source] anyhow::Error),
    /// The running application could not be described.
    #[error("could not describe the host application: {0}")]
    HostApp(String),
    /// Reading the host application's bundle or executable failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// A pending request to walk the user through one privacy pane.
#[derive(Clone)]
#[must_use = "a guide does nothing until `present` is called"]
pub struct Guide {
    panel: PrivacyPanel,
    host: Option<HostApp>,
    instruction: Option<SharedString>,
    launch_from: Option<ScreenRect>,
}

impl Guide {
    /// Starts a guide for `panel` using the running application as the one to allow.
    pub fn new(panel: PrivacyPanel) -> Self {
        Self {
            panel,
            host: None,
            instruction: None,
            launch_from: None,
        }
    }

    /// Shows `host` in the drag row instead of the running application.
    pub fn host_app(mut self, host: HostApp) -> Self {
        self.host = Some(host);
        self
    }

    /// Replaces the instruction above the drag row, "Drag *app* to the list above to allow
    /// *permission*", for example with a localized one. It also labels the drag row for
    /// assistive technologies.
    pub fn instruction(mut self, text: impl Into<SharedString>) -> Self {
        self.instruction = Some(text.into());
        self
    }

    /// Flies the overlay out of `bounds`, a rectangle in `window`'s coordinates such as the
    /// button the user just clicked. Without this the overlay appears in place.
    pub fn launch_from(mut self, bounds: Bounds<Pixels>, window: &Window) -> Self {
        self.launch_from = platform::screen_rect(window, bounds);
        self
    }

    /// Opens System Settings and shows the overlay, replacing any guide already presented.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Unsupported`] off macOS, [`Error::HostApp`] or [`Error::Io`] when the
    /// running application cannot be described, and [`Error::OpenWindow`] when GPUI cannot open
    /// the overlay window.
    pub fn present(self, cx: &mut App) -> Result<(), Error> {
        platform::ensure_supported()?;
        let host = match self.host {
            Some(host) => host,
            None => HostApp::current()?,
        };
        let instruction = self.instruction.unwrap_or_else(|| {
            format!(
                "Drag {} to the list above to allow {}",
                host.display_name,
                self.panel.title()
            )
            .into()
        });
        assistant::present(self.panel, host, instruction, self.launch_from, cx)
    }
}

/// Closes the presented guide, if any.
pub fn dismiss(cx: &mut App) {
    assistant::dismiss(cx);
}

/// Whether a guide is currently presented.
pub fn is_presented(cx: &App) -> bool {
    assistant::is_presented(cx)
}
