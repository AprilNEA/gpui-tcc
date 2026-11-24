use std::time::Duration;

use super::{
    Arrow, OVERLAY_SIZE, ScreenRect, anchored_frame, arrow_toward, launch_alpha, launch_frame,
    spring_progress,
};

const VISIBLE: ScreenRect = ScreenRect::new(0.0, 25.0, 1728.0, 1072.0);

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-6,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn anchored_frame_centers_under_settings_content_column() {
    let settings = ScreenRect::new(400.0, 200.0, 900.0, 600.0);
    let frame = anchored_frame(settings, VISIBLE);

    let content_min_x = 400.0 + 170.0;
    let content_width = 900.0 - 170.0;
    assert_close(frame.x, content_min_x + (content_width - 530.0) / 2.0 - 8.0);
    assert_close(frame.y, settings.max_y() + 10.0);
    assert_eq!((frame.width, frame.height), OVERLAY_SIZE);
}

#[test]
fn anchored_frame_stays_off_the_settings_window() {
    // Room below: the overlay sits under the window, clear of the list's + and - buttons.
    let settings = ScreenRect::new(400.0, 200.0, 900.0, 600.0);
    let below = anchored_frame(settings, VISIBLE);
    assert_close(below.intersection_area(settings), 0.0);

    // The window reaches the bottom of the screen: the overlay moves above it.
    let low = ScreenRect::new(400.0, 400.0, 900.0, 690.0);
    let above = anchored_frame(low, VISIBLE);
    assert_close(above.max_y(), low.y - 10.0);
    assert_close(above.intersection_area(low), 0.0);

    // No room above or below, as on a laptop screen: it overlaps the window's top, the pane's
    // title and description, and stays clear of the bottom, where the + and - buttons are.
    let tall = ScreenRect::new(400.0, 60.0, 900.0, 1000.0);
    let top = anchored_frame(tall, VISIBLE);
    assert_close(top.y, VISIBLE.y + 8.0);
    assert!(
        top.max_y() < tall.y + 120.0,
        "covers only the window top: {top:?}"
    );
}

#[test]
fn the_arrow_points_at_the_window_from_either_side() {
    // Below the window it points up; above it, or over its top, it points down.
    let settings = ScreenRect::new(400.0, 200.0, 900.0, 600.0);
    assert_eq!(
        arrow_toward(anchored_frame(settings, VISIBLE), settings),
        Arrow::Up
    );

    let low = ScreenRect::new(400.0, 400.0, 900.0, 690.0);
    assert_eq!(arrow_toward(anchored_frame(low, VISIBLE), low), Arrow::Down);

    let tall = ScreenRect::new(400.0, 60.0, 900.0, 1000.0);
    assert_eq!(
        arrow_toward(anchored_frame(tall, VISIBLE), tall),
        Arrow::Down
    );
}

#[test]
fn anchored_frame_clamps_inside_visible_frame() {
    let left_edge = anchored_frame(ScreenRect::new(-700.0, 100.0, 900.0, 600.0), VISIBLE);
    assert_close(left_edge.x, VISIBLE.x + 8.0);

    let right_edge = anchored_frame(ScreenRect::new(1500.0, 100.0, 900.0, 600.0), VISIBLE);
    assert_close(right_edge.max_x(), VISIBLE.max_x() - 8.0);

    let tall = anchored_frame(ScreenRect::new(400.0, -500.0, 900.0, 1700.0), VISIBLE);
    assert_close(tall.y, VISIBLE.y + 8.0);
}

#[test]
fn spring_progress_starts_at_zero_and_settles_near_one() {
    assert_close(spring_progress(Duration::ZERO), 0.0);
    let end = spring_progress(Duration::from_millis(720));
    assert!(end > 0.98 && end <= 1.0, "end progress {end}");
    let half = spring_progress(Duration::from_millis(360));
    let early = spring_progress(Duration::from_millis(100));
    assert!(early < half && half < end);
    assert_close(launch_alpha(0.0), 0.9);
    assert_close(launch_alpha(1.0), 1.0);
}

#[test]
fn launch_frame_hits_both_endpoints_and_arcs_upward() {
    let from = ScreenRect::new(100.0, 800.0, 80.0, 30.0);
    let to = ScreenRect::new(600.0, 300.0, 530.0, 109.0);

    assert_eq!(launch_frame(from, to, 0.0), from);
    let end = launch_frame(from, to, 1.0);
    assert_close(end.x, to.x);
    assert_close(end.y, to.y);
    assert_close(end.width, to.width);
    assert_close(end.height, to.height);

    // Centers: from (140, 815), to (865, 354.5); the travel is 858.9 pt, so the lift clamps to
    // 140 and the control point sits at y = 354.5 - 140 = 214.5. A quadratic Bézier's midpoint is
    // the mean of the chord midpoint (584.75) and the control point.
    let mid = launch_frame(from, to, 0.5);
    let (mid_center_x, mid_center_y) = mid.center();
    assert_close(mid_center_x, 502.5);
    assert_close(mid_center_y, f64::midpoint(584.75, 214.5));
    assert!(
        mid_center_y < 584.75,
        "midpoint {mid_center_y} should be above the straight path between the centers"
    );
}

#[test]
fn intersection_area_is_zero_for_disjoint_rects() {
    let a = ScreenRect::new(0.0, 0.0, 10.0, 10.0);
    assert_close(
        a.intersection_area(ScreenRect::new(20.0, 0.0, 10.0, 10.0)),
        0.0,
    );
    assert_close(
        a.intersection_area(ScreenRect::new(5.0, 5.0, 10.0, 10.0)),
        25.0,
    );
}
