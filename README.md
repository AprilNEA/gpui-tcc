# gpui-tcc

Walks macOS users through granting your [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) app a privacy permission. It opens the matching Privacy & Security pane in System Settings and floats a small panel over it that the user drags into the pane's app list. The panel follows the System Settings window, hides when another app comes to the front, and can fly in from the control the user clicked.

A Rust port of [Permiso](https://github.com/zats/permiso) by Sash Zats. The sample app is built on [gpui-kit](https://github.com/longbridge/gpui-kit); the library itself depends only on GPUI (`gpui-pre` 0.3.10 or any later 0.3 release), so it shares the GPUI your app already pins.

Supported panes (`PrivacyPanel`):

| Variant | System Settings pane |
| --- | --- |
| `Accessibility` | Accessibility (named "Device Control and Data Access" on macOS 27) |
| `InputMonitoring` | Input Monitoring |
| `ScreenRecording` | Screen & System Audio Recording |
| `AppManagement` | App Management |
| `FullDiskAccess` | Full Disk Access |

Requires macOS 13 or later; on other platforms `present` returns `Error::Unsupported`.

## Usage

```toml
[dependencies]
gpui-tcc = "0.1"
```

```rust
use gpui::{App, Bounds, Pixels, Window};
use gpui_tcc::{Guide, PrivacyPanel};

fn request_accessibility(button: Bounds<Pixels>, window: &Window, cx: &mut App) {
    Guide::new(PrivacyPanel::Accessibility)
        .launch_from(button, window) // optional: fly out of this control
        .present(cx)
        .expect("System Settings guide");
}
```

`gpui_tcc::dismiss(cx)` closes the panel; the panel's back button does the same. `Guide::host_app(HostApp::from_path(path)?)` shows a different app than the running one, for example a helper inside your app's bundle that holds the permission itself: the pane's `+` picker cannot browse into a bundle, but its list accepts the helper dragged from the panel.

Run the sample app with `cargo run --example sample`.
