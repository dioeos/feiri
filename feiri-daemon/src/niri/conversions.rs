pub trait IntoFeiriWindow {
    fn into_feiri_window(self) -> feiri_core::models::Window;
}
impl IntoFeiriWindow for niri_ipc::Window {
    fn into_feiri_window(self) -> feiri_core::models::Window {
        feiri_core::models::Window {
            id: feiri_core::models::WindowId(self.id),
            title: self.title,
            app_id: self.app_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use niri_ipc::WindowLayout;

    #[test]
    fn converts_niri_window_to_niqol_window() {
        let niri_window = niri_ipc::Window {
            id: 42,
            title: Some("Ghostty".to_owned()),
            app_id: Some("com.mitchellh.ghostty".to_owned()),
            pid: Some(5),
            workspace_id: Some(2),
            is_focused: false,
            is_floating: false,
            is_urgent: false,
            layout: WindowLayout {
                pos_in_scrolling_layout: None,
                tile_size: (0.0, 0.0),
                window_size: (0, 0),
                tile_pos_in_workspace_view: None,
                window_offset_in_tile: (0.0, 0.0),
            },
            focus_timestamp: None,
        };

        let window = niri_window.into_feiri_window();

        assert_eq!(window.id, feiri_core::models::WindowId(42));
        assert_eq!(window.title.as_deref(), Some("Ghostty"));
        assert_eq!(window.app_id.as_deref(), Some("com.mitchellh.ghostty"));
    }

    #[test]
    fn preserves_none_optional_fields() {
        let niri_window = niri_ipc::Window {
            id: 42,
            title: None,
            app_id: None,
            pid: Some(5),
            workspace_id: Some(2),
            is_focused: false,
            is_floating: false,
            is_urgent: false,
            layout: WindowLayout {
                pos_in_scrolling_layout: None,
                tile_size: (0.0, 0.0),
                window_size: (0, 0),
                tile_pos_in_workspace_view: None,
                window_offset_in_tile: (0.0, 0.0),
            },
            focus_timestamp: None,
        };

        let window = niri_window.into_feiri_window();

        assert_eq!(window.id, feiri_core::models::WindowId(42));
        assert_eq!(window.title, None);
        assert_eq!(window.app_id, None);
    }
}
