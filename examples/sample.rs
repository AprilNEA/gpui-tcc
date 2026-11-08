//! A window with one button per privacy pane. Each button flies the guide out of its own frame.

use gpui_kit::component::StyledExt as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::prelude::*;
use gpui_kit::{
    App, Bounds, Context, Entity, Pixels, SharedString, TitlebarOptions, Window, WindowBounds,
    WindowOptions, div, px, size,
};
use gpui_tcc::{Guide, PrivacyPanel};

const PANELS: [PrivacyPanel; 5] = [
    PrivacyPanel::Accessibility,
    PrivacyPanel::InputMonitoring,
    PrivacyPanel::ScreenRecording,
    PrivacyPanel::AppManagement,
    PrivacyPanel::FullDiskAccess,
];

struct Sample {
    /// Last painted frame of each button in `PANELS` order, in window coordinates.
    buttons: [Option<Bounds<Pixels>>; PANELS.len()],
    status: SharedString,
}

impl Sample {
    fn request(&mut self, index: usize, window: &Window, cx: &mut App) {
        let panel = PANELS[index];
        let mut guide = Guide::new(panel);
        if let Some(bounds) = self.buttons[index] {
            guide = guide.launch_from(bounds, window);
        }
        self.status = match guide.present(cx) {
            Ok(()) => format!("Guiding to {}", panel.title()).into(),
            Err(error) => format!("{error:#}").into(),
        };
    }

    fn panel_button(index: usize, this: &Entity<Self>) -> impl IntoElement {
        let panel = PANELS[index];
        let on_prepaint = this.clone();
        let on_click = this.clone();
        div()
            .on_children_prepainted(move |bounds, _, cx| {
                on_prepaint.update(cx, |sample, _| {
                    sample.buttons[index] = bounds.first().copied();
                });
            })
            .child(
                Button::new(panel.title())
                    .w(px(260.))
                    .primary()
                    .label(format!("Request {}", panel.title()))
                    .on_click(move |_, window, cx| {
                        on_click.update(cx, |sample, cx| {
                            sample.request(index, window, cx);
                            cx.notify();
                        });
                    }),
            )
    }
}

impl Render for Sample {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let this = cx.entity();
        let dismiss = this.clone();
        div()
            .v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .gap_3()
            .children((0..PANELS.len()).map(|index| Self::panel_button(index, &this)))
            .child(
                Button::new("dismiss")
                    .w(px(260.))
                    .outline()
                    .label("Dismiss guide")
                    .on_click(move |_, _, cx| {
                        gpui_tcc::dismiss(cx);
                        dismiss.update(cx, |sample, cx| {
                            sample.status = "Dismissed".into();
                            cx.notify();
                        });
                    }),
            )
            .child(div().text_sm().child(self.status.clone()))
    }
}

fn main() {
    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(420.), px(320.)),
                cx,
            ))),
            titlebar: Some(TitlebarOptions {
                title: Some("gpui-tcc sample".into()),
                ..TitlebarOptions::default()
            }),
            ..WindowOptions::default()
        };
        gpui_kit::open_window(options, cx, |_, cx| {
            cx.new(|_| Sample {
                buttons: [None; PANELS.len()],
                status: "Pick a permission to request".into(),
            })
        })
        .expect("open the sample window");
        cx.activate(true);
    });
}
