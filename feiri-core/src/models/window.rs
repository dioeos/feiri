use serde::{Deserialize, Serialize};

/// Metadata describing a window known to Feiri.
///
/// The struct is not equivalent to the window within Niri; however, the information
/// is sourced from a Niri window.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Window {
    /// Unique identifier assigned to the window
    pub id: WindowId,

    /// Human-readable window title, if available.
    /// For example: `"README.md - Feiri GitHub"` or `"YouTube - Mozilla FireFox"`
    pub title: Option<String>,

    /// Application identifier associated with the window according to Niri, if available
    /// For example: `"org.mozilla.firefox"` or `"com.mitchellh.ghostty"`
    pub app_id: Option<String>,
}

/// Unique Feiri-domain specific identifier that wraps a window's ID assigned by Niri.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowId(pub u64);
