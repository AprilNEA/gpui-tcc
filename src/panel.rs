/// A System Settings privacy pane that lists apps the user can allow by dragging them in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrivacyPanel {
    /// Privacy & Security › Accessibility.
    Accessibility,
    /// Privacy & Security › Input Monitoring.
    InputMonitoring,
    /// Privacy & Security › Screen & System Audio Recording.
    ScreenRecording,
    /// Privacy & Security › App Management.
    AppManagement,
    /// Privacy & Security › Full Disk Access.
    FullDiskAccess,
}

impl PrivacyPanel {
    /// The `x-apple.systempreferences:` deep link that opens this pane.
    pub fn settings_url(self) -> String {
        let anchor = match self {
            Self::Accessibility => "Privacy_Accessibility",
            Self::InputMonitoring => "Privacy_ListenEvent",
            Self::ScreenRecording => "Privacy_ScreenCapture",
            Self::AppManagement => "Privacy_AppBundles",
            Self::FullDiskAccess => "Privacy_AllFiles",
        };
        format!("x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?{anchor}")
    }

    /// The permission name as System Settings labels it.
    pub fn title(self) -> &'static str {
        match self {
            Self::Accessibility => "Accessibility",
            Self::InputMonitoring => "Input Monitoring",
            Self::ScreenRecording => "Screen Recording",
            Self::AppManagement => "App Management",
            Self::FullDiskAccess => "Full Disk Access",
        }
    }
}
