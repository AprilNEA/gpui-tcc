//! Opens the overlay window, follows the System Settings window, and runs the launch animation.

use std::time::{Duration, Instant};

use gpui::{
    App, AppContext as _, AsyncApp, Bounds, Global, Task, WindowBackgroundAppearance, WindowBounds,
    WindowHandle, WindowKind, WindowOptions, point, px, size,
};

use crate::Error;
use crate::host_app::HostApp;
use crate::motion::{self, INITIAL_ALPHA, LAUNCH_DURATION, OVERLAY_SIZE, ScreenRect};
use crate::overlay::Overlay;
use crate::panel::PrivacyPanel;
use crate::platform::{self, NativeOverlay};

/// How often the overlay re-anchors to the System Settings window.
const TRACK_INTERVAL: Duration = Duration::from_millis(150);

/// Frame interval of the launch animation.
const ANIMATION_INTERVAL: Duration = Duration::from_millis(8);

/// The one overlay the application shows at a time, stored as a GPUI global.
#[derive(Default)]
pub(crate) struct Assistant {
    session: Option<Session>,
}

impl Global for Assistant {}

struct Session {
    overlay: WindowHandle<Overlay>,
    native: NativeOverlay,
    phase: Phase,
    _tracker: Task<()>,
}

#[derive(Debug, Clone, Copy)]
enum Phase {
    /// System Settings has not come to the front since `present` ran.
    Pending { launch_from: Option<ScreenRect> },
    /// The overlay is flying from `from` to its anchored frame.
    Launching { started: Instant, from: ScreenRect },
    /// The overlay rests at its anchored frame and follows the System Settings window.
    Tracking,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Placement {
    Hidden,
    Shown { frame: ScreenRect, alpha: f64 },
}

pub(crate) fn present(
    panel: PrivacyPanel,
    host: HostApp,
    launch_from: Option<ScreenRect>,
    cx: &mut App,
) -> Result<(), Error> {
    platform::ensure_supported()?;
    dismiss(cx);

    let (width, height) = OVERLAY_SIZE;
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::new(
            point(px(0.), px(0.)),
            size(px(width as f32), px(height as f32)),
        ))),
        titlebar: None,
        focus: false,
        show: false,
        kind: WindowKind::PopUp,
        is_movable: false,
        is_resizable: false,
        is_minimizable: false,
        window_background: WindowBackgroundAppearance::Transparent,
        ..WindowOptions::default()
    };
    let mut native = None;
    let overlay = cx
        .open_window(options, |window, cx| {
            native = platform::prepare_overlay(window);
            cx.new(|_| Overlay::new(panel, host))
        })
        .map_err(Error::OpenWindow)?;
    let Some(native) = native else {
        overlay
            .update(cx, |_, window, _| window.remove_window())
            .ok();
        return Err(Error::OpenWindow(anyhow::anyhow!(
            "the overlay window has no native window"
        )));
    };
    cx.open_url(&panel.settings_url());

    // AppKit reports each native move and resize back to GPUI through a callback that needs
    // to borrow the app, so the overlay is placed between updates rather than inside one.
    let tracker = cx.spawn(async move |cx: &mut AsyncApp| {
        loop {
            let Some((native, placement, interval)) = cx.update(advance) else {
                return;
            };
            match placement {
                Placement::Hidden => native.hide(),
                Placement::Shown { frame, alpha } => native.place(frame, alpha),
            }
            cx.background_executor().timer(interval).await;
        }
    });
    cx.set_global(Assistant {
        session: Some(Session {
            overlay,
            native,
            phase: Phase::Pending { launch_from },
            _tracker: tracker,
        }),
    });
    Ok(())
}

pub(crate) fn dismiss(cx: &mut App) {
    if !cx.has_global::<Assistant>() {
        return;
    }
    let assistant = cx.remove_global::<Assistant>();
    if let Some(session) = assistant.session {
        session
            .overlay
            .update(cx, |_, window, _| window.remove_window())
            .ok();
    }
}

pub(crate) fn is_presented(cx: &App) -> bool {
    cx.try_global::<Assistant>()
        .is_some_and(|assistant| assistant.session.is_some())
}

/// Advances the session one step and returns what to do with the native window and how long to
/// wait before the next step, or `None` once the overlay is gone.
fn advance(cx: &mut App) -> Option<(NativeOverlay, Placement, Duration)> {
    if !cx.has_global::<Assistant>() {
        return None;
    }
    let session = cx.global_mut::<Assistant>().session.as_mut()?;
    let (placement, phase, interval) = step(session.phase, platform::settings_window());
    session.phase = phase;
    Some((session.native.clone(), placement, interval))
}

fn step(phase: Phase, settings: Option<platform::SettingsWindow>) -> (Placement, Phase, Duration) {
    let Some(settings) = settings else {
        let phase = match phase {
            Phase::Launching { .. } => Phase::Tracking,
            other => other,
        };
        return (Placement::Hidden, phase, TRACK_INTERVAL);
    };

    let target = motion::anchored_frame(settings.frame, settings.visible);
    let resting = Placement::Shown {
        frame: target,
        alpha: 1.0,
    };
    match phase {
        Phase::Pending {
            launch_from: Some(from),
        } if !from.is_empty() => (
            Placement::Shown {
                frame: from,
                alpha: INITIAL_ALPHA,
            },
            Phase::Launching {
                started: Instant::now(),
                from,
            },
            ANIMATION_INTERVAL,
        ),
        Phase::Pending { .. } | Phase::Tracking => (resting, Phase::Tracking, TRACK_INTERVAL),
        Phase::Launching { started, from } => {
            let elapsed = started.elapsed();
            if elapsed >= LAUNCH_DURATION {
                return (resting, Phase::Tracking, TRACK_INTERVAL);
            }
            let progress = motion::spring_progress(elapsed);
            (
                Placement::Shown {
                    frame: motion::launch_frame(from, target, progress),
                    alpha: motion::launch_alpha(progress),
                },
                phase,
                ANIMATION_INTERVAL,
            )
        }
    }
}
