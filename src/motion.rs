//! Overlay placement and launch-animation geometry.
//!
//! All rectangles are in global screen points with the origin at the top-left corner of the
//! primary display (Core Graphics convention), so they compose with GPUI's top-left layout.

#![cfg_attr(
    not(target_os = "macos"),
    allow(
        dead_code,
        reason = "the macOS platform layer is the only caller of the screen-geometry helpers"
    )
)]

use std::f64::consts::PI;
use std::time::Duration;

/// Size of the overlay window in points.
pub const OVERLAY_SIZE: (f64, f64) = (530.0, 109.0);

/// How long the launch animation runs; it also sets the spring response.
pub const LAUNCH_DURATION: Duration = Duration::from_millis(720);

/// Window opacity when the launch animation starts.
pub const INITIAL_ALPHA: f64 = 0.9;

/// Width of the System Settings sidebar the overlay is centered against.
const SIDEBAR_WIDTH: f64 = 170.0;

/// Margin kept between the overlay and the screen's visible frame.
const SCREEN_MARGIN: f64 = 8.0;

/// A rectangle in global screen points, origin at the top-left of the primary display.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ScreenRect {
    /// Left edge.
    pub x: f64,
    /// Top edge.
    pub y: f64,
    /// Width in points.
    pub width: f64,
    /// Height in points.
    pub height: f64,
}

impl ScreenRect {
    /// Creates a rectangle from its top-left corner and size.
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Bottom edge.
    pub fn max_y(self) -> f64 {
        self.y + self.height
    }

    /// Right edge.
    pub fn max_x(self) -> f64 {
        self.x + self.width
    }

    /// Center point as `(x, y)`.
    pub fn center(self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Whether the rectangle has no area.
    pub fn is_empty(self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    /// The rectangle shrunk by `dx` on each horizontal side and `dy` on each vertical side.
    pub fn inset(self, dx: f64, dy: f64) -> Self {
        Self::new(
            self.x + dx,
            self.y + dy,
            self.width - 2.0 * dx,
            self.height - 2.0 * dy,
        )
    }

    /// Area of the overlap between two rectangles, or zero when they do not touch.
    pub fn intersection_area(self, other: Self) -> f64 {
        let width = self.max_x().min(other.max_x()) - self.x.max(other.x);
        let height = self.max_y().min(other.max_y()) - self.y.max(other.y);
        if width <= 0.0 || height <= 0.0 {
            0.0
        } else {
            width * height
        }
    }
}

/// Where the overlay sits relative to the System Settings window: horizontally centered under
/// the content column (right of the sidebar), resting 14 pt above the window's bottom edge, and
/// clamped to stay [`SCREEN_MARGIN`] inside the screen's visible frame.
pub fn anchored_frame(settings: ScreenRect, visible: ScreenRect) -> ScreenRect {
    let (width, height) = OVERLAY_SIZE;
    let content_min_x = settings.x + SIDEBAR_WIDTH;
    let content_width = (settings.width - SIDEBAR_WIDTH).max(width);
    let preferred_x = content_min_x + (content_width - width) / 2.0 - 8.0;
    let preferred_y = settings.max_y() - 14.0 - height;

    let min_x = visible.x + SCREEN_MARGIN;
    let max_x = visible.max_x() - width - SCREEN_MARGIN;
    let min_y = visible.y + SCREEN_MARGIN;
    let max_y = visible.max_y() - height - SCREEN_MARGIN;

    ScreenRect::new(
        preferred_x.clamp(min_x, max_x),
        preferred_y.clamp(min_y, max_y),
        width,
        height,
    )
}

/// The frame to anchor against when System Settings is frontmost but no window is visible yet.
pub fn fallback_settings_frame(visible: ScreenRect) -> ScreenRect {
    visible.inset(80.0, 70.0)
}

/// Progress of a critically damped spring with response [`LAUNCH_DURATION`], clamped to `0..=1`.
pub fn spring_progress(elapsed: Duration) -> f64 {
    let omega = 2.0 * PI / LAUNCH_DURATION.as_secs_f64();
    let t = elapsed.as_secs_f64();
    (1.0 - (-omega * t).exp() * (1.0 + omega * t)).clamp(0.0, 1.0)
}

/// Window opacity at the given launch progress.
pub fn launch_alpha(progress: f64) -> f64 {
    INITIAL_ALPHA + (1.0 - INITIAL_ALPHA) * progress
}

/// The frame at `progress` along the launch path from `from` to `to`.
///
/// The size interpolates linearly. The center follows a quadratic Bézier whose control point
/// sits above the higher of the two centers, lifted by 18% of the travel distance (44–140 pt),
/// so the overlay arcs upward before settling.
pub fn launch_frame(from: ScreenRect, to: ScreenRect, progress: f64) -> ScreenRect {
    let width = from.width + (to.width - from.width) * progress;
    let height = from.height + (to.height - from.height) * progress;

    let (start_x, start_y) = from.center();
    let (end_x, end_y) = to.center();
    let distance = (end_x - start_x).hypot(end_y - start_y);
    let lift = (distance * 0.18).clamp(44.0, 140.0);
    let control_x = start_x.midpoint(end_x);
    let control_y = start_y.min(end_y) - lift;

    let inverse = 1.0 - progress;
    let center_x = inverse * inverse * start_x
        + 2.0 * inverse * progress * control_x
        + progress * progress * end_x;
    let center_y = inverse * inverse * start_y
        + 2.0 * inverse * progress * control_y
        + progress * progress * end_y;

    ScreenRect::new(
        center_x - width / 2.0,
        center_y - height / 2.0,
        width,
        height,
    )
}

#[cfg(test)]
mod tests;
