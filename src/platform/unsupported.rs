use gpui_kit::{Bounds, Pixels, Window};

use super::{HostAppInfo, SettingsWindow};
use crate::Error;
use crate::motion::ScreenRect;

pub fn ensure_supported() -> Result<(), Error> {
    Err(Error::Unsupported)
}

pub fn host_app_info() -> Result<HostAppInfo, Error> {
    Err(Error::Unsupported)
}

pub fn settings_window() -> Option<SettingsWindow> {
    None
}

pub fn screen_rect(_window: &Window, _bounds: Bounds<Pixels>) -> Option<ScreenRect> {
    None
}

pub fn prepare_overlay(_window: &Window) {}

pub fn place_overlay(_window: &Window, _frame: ScreenRect, _alpha: f64) {}

pub fn hide_overlay(_window: &Window) {}
