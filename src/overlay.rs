//! The overlay content: back button, instruction, and the row the user drags into System Settings.

use std::sync::Arc;

use gpui_kit::prelude::*;
use gpui_kit::{
    Context, ExternalDragPayload, FileDragPaths, FontWeight, Image, ImageFormat, Rgba, Role,
    Window, WindowAppearance, div, img, px,
};

use crate::host_app::HostApp;
use crate::motion::OVERLAY_SIZE;
use crate::panel::PrivacyPanel;

/// Font family resolved by the macOS text system to the system UI font.
const SYSTEM_FONT: &str = ".SystemUIFont";

/// Intrinsic-size multiplier for inline SVGs, which GPUI rasterizes at scale 1.
const ICON_RASTER_SCALE: f64 = 3.0;

const ARROW_BLUE: Rgba = Rgba {
    r: 0.15,
    g: 0.54,
    b: 0.98,
    a: 1.0,
};

const ICON_CHROME: Rgba = Rgba {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 0.9,
};

pub(crate) struct Overlay {
    panel: PrivacyPanel,
    host: HostApp,
}

impl Overlay {
    pub(crate) fn new(panel: PrivacyPanel, host: HostApp) -> Self {
        Self { panel, host }
    }
}

impl Render for Overlay {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::for_appearance(window.appearance());
        let (width, height) = OVERLAY_SIZE;
        let title = format!(
            "Drag {} to the list above to allow {}",
            self.host.display_name,
            self.panel.title()
        );

        div()
            .relative()
            .w(px(width as f32))
            .h(px(height as f32))
            .rounded(px(18.))
            .bg(palette.background)
            .border(px(0.5))
            .border_color(palette.border)
            .font_family(SYSTEM_FONT)
            .text_color(palette.label)
            .child(
                img(icon_svg(ARROW_UP, ARROW_BLUE, 28.0))
                    .absolute()
                    .left(px(35.))
                    .top(px(10.))
                    .size(px(28.)),
            )
            .child(
                div()
                    .absolute()
                    .left(px(73.))
                    .right(px(22.))
                    .top(px(10.))
                    .h(px(28.))
                    .flex()
                    .items_center()
                    .text_size(px(14.))
                    .font_weight(FontWeight::MEDIUM)
                    .truncate()
                    .child(title),
            )
            .child(
                div()
                    .id("back")
                    .role(Role::Button)
                    .aria_label("Back")
                    .absolute()
                    .left(px(18.))
                    .top(px(52.))
                    .size(px(32.))
                    .rounded_full()
                    .bg(palette.control_background)
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .on_click(|_, _, cx| cx.defer(crate::dismiss))
                    .child(img(icon_svg(CHEVRON_LEFT, palette.chevron, 14.0)).size(px(14.))),
            )
            .child(
                div()
                    .id("drag-row")
                    .role(Role::Group)
                    .aria_label(format!(
                        "Drag {} into System Settings",
                        self.host.display_name
                    ))
                    .absolute()
                    .left(px(64.))
                    .right(px(21.))
                    .top(px(47.))
                    .h(px(43.))
                    .rounded(px(7.))
                    .bg(palette.row_background)
                    .border_1()
                    .border_color(palette.row_border)
                    .flex()
                    .items_center()
                    .pl(px(10.))
                    .pr(px(12.))
                    .gap(px(11.))
                    .cursor_grab()
                    .on_drag(self.host.clone(), |host, _, _, cx| {
                        let icon = host.icon.clone();
                        cx.new(|_| DragPreview { icon })
                    })
                    .external_drag_payload(|host: &HostApp, _, _| {
                        Some(ExternalDragPayload::Files(FileDragPaths::new([(
                            host.path.clone(),
                            false,
                        )])))
                    })
                    .child(icon_chrome(self.host.icon.clone()))
                    .child(
                        div()
                            .text_size(px(15.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .truncate()
                            .child(self.host.display_name.clone()),
                    ),
            )
    }
}

/// What follows the pointer while the drag is still inside the overlay window.
struct DragPreview {
    icon: Option<Arc<Image>>,
}

impl Render for DragPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        icon_chrome(self.icon.clone())
    }
}

fn icon_chrome(icon: Option<Arc<Image>>) -> impl IntoElement {
    div()
        .flex_none()
        .size(px(26.))
        .rounded(px(6.))
        .bg(ICON_CHROME)
        .flex()
        .items_center()
        .justify_center()
        .children(icon.map(|icon| img(icon).size(px(22.))))
}

/// Colors matching the `AppKit` semantic colors Permiso uses, resolved per appearance.
struct Palette {
    background: Rgba,
    border: Rgba,
    label: Rgba,
    control_background: Rgba,
    chevron: Rgba,
    row_background: Rgba,
    row_border: Rgba,
}

impl Palette {
    fn for_appearance(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Light | WindowAppearance::VibrantLight => Self {
                background: gray(0.925, 0.96),
                border: gray(0.0, 0.08),
                label: gray(0.0, 0.70),
                control_background: gray(1.0, 0.95),
                chevron: gray(0.0, 0.61),
                row_background: gray(1.0, 0.65),
                row_border: Rgba {
                    r: 0.8745,
                    g: 0.8667,
                    b: 0.8627,
                    a: 1.0,
                },
            },
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Self {
                background: gray(0.196, 0.96),
                border: gray(1.0, 0.08),
                label: gray(1.0, 0.70),
                control_background: gray(0.118, 0.95),
                chevron: gray(1.0, 0.61),
                row_background: gray(1.0, 0.06),
                row_border: gray(1.0, 0.08),
            },
        }
    }
}

const fn gray(level: f32, alpha: f32) -> Rgba {
    Rgba {
        r: level,
        g: level,
        b: level,
        a: alpha,
    }
}

/// A stroked icon drawn on a 24-unit grid, standing in for an SF Symbol.
#[derive(Clone, Copy)]
struct Icon {
    paths: &'static str,
    stroke_width: f64,
}

const ARROW_UP: Icon = Icon {
    paths: r#"<path d="M12 20.5V4.5"/><path d="M5 11.5l7-7 7 7"/>"#,
    stroke_width: 3.6,
};

const CHEVRON_LEFT: Icon = Icon {
    paths: r#"<path d="M15 5l-7 7 7 7"/>"#,
    stroke_width: 2.8,
};

/// Rasterizes `icon` in `color` for a `display_size` point box.
fn icon_svg(icon: Icon, color: Rgba, display_size: f64) -> Arc<Image> {
    let svg = format!(
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 24 24" "#,
            r##"fill="none" stroke="#{rgba:08x}" stroke-width="{stroke}" "##,
            r#"stroke-linecap="round" stroke-linejoin="round">{paths}</svg>"#
        ),
        size = display_size * ICON_RASTER_SCALE,
        rgba = u32::from(color),
        stroke = icon.stroke_width,
        paths = icon.paths,
    );
    Arc::new(Image::from_bytes(ImageFormat::Svg, svg.into_bytes()))
}
