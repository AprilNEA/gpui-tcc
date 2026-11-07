use std::path::{Path, PathBuf};

use gpui_kit::{Bounds, Pixels, Window};
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{
    NSBitmapImageFileType, NSBitmapImageRep, NSScreen, NSView, NSWindow, NSWindowAnimationBehavior,
    NSWorkspace,
};
use objc2_core_foundation::{
    CFArray, CFDictionary, CFNumber, CFRetained, CFString, CFType, CGRect,
};
use objc2_core_graphics::{
    CGRectMakeWithDictionaryRepresentation, CGWindowListCopyWindowInfo, CGWindowListOption,
    kCGNullWindowID, kCGWindowBounds, kCGWindowLayer, kCGWindowOwnerPID,
};
use objc2_foundation::{NSBundle, NSDictionary, NSPoint, NSRect, NSSize, NSString};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

use super::{HostAppInfo, SettingsWindow};
use crate::Error;
use crate::motion::{self, ScreenRect};

const SETTINGS_BUNDLE_ID: &str = "com.apple.systempreferences";

/// Windows smaller than this are System Settings auxiliary panels, not the main window.
const MIN_SETTINGS_WINDOW_SIZE: (f64, f64) = (320.0, 240.0);

/// Side length in points of the icon bitmap handed to GPUI.
const ICON_SIZE: f64 = 128.0;

pub fn ensure_supported() -> Result<(), Error> {
    Ok(())
}

pub fn host_app_info() -> Result<HostAppInfo, Error> {
    let bundle = NSBundle::mainBundle();
    let bundle_path = PathBuf::from(bundle.bundlePath().to_string());
    let path = if bundle_path.extension().is_some_and(|ext| ext == "app") {
        bundle_path
    } else {
        std::env::current_exe()?
    };
    let display_name = ["CFBundleDisplayName", "CFBundleName"]
        .into_iter()
        .find_map(|key| info_string(&bundle, key))
        .or_else(|| {
            path.file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
        })
        .ok_or_else(|| Error::HostApp(format!("no name for {}", path.display())))?;
    let icon_png = icon_png(&path);
    Ok(HostAppInfo {
        display_name,
        path,
        icon_png,
    })
}

fn info_string(bundle: &NSBundle, key: &str) -> Option<String> {
    let value = bundle.objectForInfoDictionaryKey(&NSString::from_str(key))?;
    let value = value.downcast::<NSString>().ok()?;
    Some(value.to_string()).filter(|value| !value.is_empty())
}

fn icon_png(path: &Path) -> Option<Vec<u8>> {
    let icon =
        NSWorkspace::sharedWorkspace().iconForFile(&NSString::from_str(&path.to_string_lossy()));
    let mut rect = NSRect::new(NSPoint::ZERO, NSSize::new(ICON_SIZE, ICON_SIZE));
    // SAFETY: `rect` is a valid, writable pointer for the call and `hints` is `None`.
    let cg_image = unsafe { icon.CGImageForProposedRect_context_hints(&mut rect, None, None) }?;
    let rep = NSBitmapImageRep::initWithCGImage(NSBitmapImageRep::alloc(), &cg_image);
    // SAFETY: An empty property dictionary matches the declared key and value types.
    let data = unsafe {
        rep.representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new())
    }?;
    Some(data.to_vec())
}

pub fn settings_window() -> Option<SettingsWindow> {
    let mtm = MainThreadMarker::new()?;
    let frontmost = NSWorkspace::sharedWorkspace().frontmostApplication()?;
    if frontmost.bundleIdentifier()?.to_string() != SETTINGS_BUNDLE_ID {
        return None;
    }
    let screens = Screens::current(mtm)?;
    match largest_window_frame(frontmost.processIdentifier()) {
        Some(frame) => Some(SettingsWindow {
            frame,
            visible: screens.visible_frame_for(frame),
        }),
        None => {
            let visible = screens.main_visible_frame(mtm);
            Some(SettingsWindow {
                frame: motion::fallback_settings_frame(visible),
                visible,
            })
        }
    }
}

fn largest_window_frame(pid: i32) -> Option<ScreenRect> {
    let infos = CGWindowListCopyWindowInfo(
        CGWindowListOption::OptionOnScreenOnly | CGWindowListOption::ExcludeDesktopElements,
        kCGNullWindowID,
    )?;
    // SAFETY: `CGWindowListCopyWindowInfo` returns an array of dictionaries keyed by the
    // `kCGWindow*` CFString constants.
    let infos: CFRetained<CFArray<CFDictionary<CFString, CFType>>> =
        unsafe { CFRetained::cast_unchecked(infos) };
    infos
        .iter()
        .filter_map(|info| {
            // SAFETY: These are immutable constants exported by CoreGraphics.
            let (owner_key, layer_key, bounds_key) =
                unsafe { (kCGWindowOwnerPID, kCGWindowLayer, kCGWindowBounds) };
            if number(&info, owner_key)?.as_i32()? != pid
                || number(&info, layer_key)?.as_i32()? != 0
            {
                return None;
            }
            let bounds = info.get(bounds_key)?.downcast::<CFDictionary>().ok()?;
            let mut rect = CGRect::ZERO;
            // SAFETY: `rect` is a valid, writable pointer for the duration of the call.
            if !unsafe { CGRectMakeWithDictionaryRepresentation(Some(&bounds), &mut rect) } {
                return None;
            }
            let frame = ScreenRect::new(
                rect.origin.x,
                rect.origin.y,
                rect.size.width,
                rect.size.height,
            );
            let (min_width, min_height) = MIN_SETTINGS_WINDOW_SIZE;
            (frame.width > min_width && frame.height > min_height).then_some(frame)
        })
        .max_by(|a, b| (a.width * a.height).total_cmp(&(b.width * b.height)))
}

fn number(info: &CFDictionary<CFString, CFType>, key: &CFString) -> Option<CFRetained<CFNumber>> {
    info.get(key)?.downcast::<CFNumber>().ok()
}

pub fn screen_rect(window: &Window, bounds: Bounds<Pixels>) -> Option<ScreenRect> {
    let mtm = MainThreadMarker::new()?;
    let view = ns_view(window)?;
    let ns_window = view.window()?;
    let view_height = view.bounds().size.height;
    let (width, height) = (f64::from(bounds.size.width), f64::from(bounds.size.height));
    let in_view = NSRect::new(
        NSPoint::new(
            f64::from(bounds.origin.x),
            view_height - (f64::from(bounds.origin.y) + height),
        ),
        NSSize::new(width, height),
    );
    let in_window = view.convertRect_toView(in_view, None);
    let on_screen = ns_window.convertRectToScreen(in_window);
    Some(flip(on_screen, Screens::current(mtm)?.primary_height))
}

pub fn prepare_overlay(window: &Window) {
    if let Some(ns_window) = ns_window(window) {
        ns_window.setHidesOnDeactivate(false);
        ns_window.setAnimationBehavior(NSWindowAnimationBehavior::None);
    }
}

pub fn place_overlay(window: &Window, frame: ScreenRect, alpha: f64) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let Some((ns_window, screens)) = ns_window(window).zip(Screens::current(mtm)) else {
        return;
    };
    ns_window.setFrame_display(unflip(frame, screens.primary_height), false);
    ns_window.setAlphaValue(alpha);
    if !ns_window.isVisible() {
        ns_window.orderFrontRegardless();
    }
}

pub fn hide_overlay(window: &Window) {
    if let Some(ns_window) = ns_window(window) {
        ns_window.orderOut(None);
    }
}

fn ns_window(window: &Window) -> Option<Retained<NSWindow>> {
    ns_view(window)?.window()
}

fn ns_view(window: &Window) -> Option<Retained<NSView>> {
    let handle = window.window_handle().ok()?;
    let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
        return None;
    };
    // SAFETY: GPUI reports the pointer of the live NSView backing this window.
    unsafe { Retained::retain(handle.ns_view.as_ptr().cast::<NSView>()) }
}

/// The attached screens in top-left global coordinates.
struct Screens {
    /// Height of the primary display; AppKit measures y upward from its bottom edge.
    primary_height: f64,
    /// `(frame, visible_frame)` per screen.
    frames: Vec<(ScreenRect, ScreenRect)>,
}

impl Screens {
    fn current(mtm: MainThreadMarker) -> Option<Self> {
        let screens = NSScreen::screens(mtm);
        let primary_height = screens.firstObject()?.frame().size.height;
        let frames = screens
            .iter()
            .map(|screen| {
                (
                    flip(screen.frame(), primary_height),
                    flip(screen.visibleFrame(), primary_height),
                )
            })
            .collect();
        Some(Self {
            primary_height,
            frames,
        })
    }

    /// Visible frame of the screen that shows most of `frame`, or of the primary display.
    fn visible_frame_for(&self, frame: ScreenRect) -> ScreenRect {
        self.frames
            .iter()
            .map(|(screen, visible)| (screen.intersection_area(frame), *visible))
            .filter(|(area, _)| *area > 0.0)
            .max_by(|a, b| a.0.total_cmp(&b.0))
            .map_or(self.frames[0].1, |(_, visible)| visible)
    }

    fn main_visible_frame(&self, mtm: MainThreadMarker) -> ScreenRect {
        NSScreen::mainScreen(mtm).map_or(self.frames[0].1, |screen| {
            flip(screen.visibleFrame(), self.primary_height)
        })
    }
}

/// Converts AppKit screen coordinates (y up from the primary display's bottom edge) to top-left
/// global coordinates.
fn flip(rect: NSRect, primary_height: f64) -> ScreenRect {
    ScreenRect::new(
        rect.origin.x,
        primary_height - (rect.origin.y + rect.size.height),
        rect.size.width,
        rect.size.height,
    )
}

/// Inverse of [`flip`].
fn unflip(rect: ScreenRect, primary_height: f64) -> NSRect {
    NSRect::new(
        NSPoint::new(rect.x, primary_height - (rect.y + rect.height)),
        NSSize::new(rect.width, rect.height),
    )
}
