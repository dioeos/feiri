use std::{fs, path::PathBuf};

use super::{discover::data_dirs, error::Error};

pub struct DesktopEntry {
    desktop_id: String,
    icon: Option<String>,
    startup_wm_class: Option<String>,
}

impl DesktopEntry {
    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    pub fn new(desktop_id: String) -> Self {
        Self {
            desktop_id,
            icon: None,
            startup_wm_class: None,
        }
    }
}

pub struct DesktopEntryIndex {
    entries: Vec<DesktopEntry>,
}

impl DesktopEntryIndex {
    pub fn build() -> Result<Self, Error> {
        let dirs = data_dirs();
        let mut entries = Vec::new();

        let apps = dirs
            .iter()
            .map(|dir| dir.join("applications"))
            .collect::<Vec<PathBuf>>();

        for apps_dir in &apps {
            if !apps_dir.is_dir() {
                continue;
            }

            for entry in fs::read_dir(apps_dir)? {
                let path = entry?.path();

                if path.extension().is_some_and(|ext| ext == "desktop") {
                    let Some(desktop_app_name) = path.file_stem().and_then(|s| s.to_str()) else {
                        continue;
                    };

                    let mut entry = DesktopEntry::new(desktop_app_name.to_owned());

                    let contents = fs::read_to_string(path)?;

                    let mut in_desktop_entry = false;
                    for line in contents.lines().map(str::trim) {
                        if line.starts_with('[') {
                            in_desktop_entry = line == "[Desktop Entry]";
                            continue;
                        }

                        if !in_desktop_entry {
                            continue;
                        }

                        if let Some(wm_class) = line.strip_prefix("StartupWMClass=")
                            && !wm_class.is_empty()
                        {
                            entry.startup_wm_class = Some(wm_class.to_owned());
                        }

                        if let Some(icon) = line.strip_prefix("Icon=")
                            && !icon.is_empty()
                        {
                            entry.icon = Some(icon.to_owned());
                        }
                    }
                    entries.push(entry);
                }
            }
        }

        Ok(Self { entries })
    }
}

pub fn find_entry_for_app_id<'a>(
    index: &'a DesktopEntryIndex,
    app_id: &Option<String>,
) -> Option<&'a DesktopEntry> {
    fn normalize(id: &str) -> &str {
        id.strip_suffix(".desktop").unwrap_or(id)
    }

    fn last_segment(id: &str) -> &str {
        id.rsplit('.').next().unwrap_or(id)
    }

    let app_id = normalize(app_id.as_deref()?);
    if app_id.is_empty() {
        return None;
    }

    let entries = &index.entries;

    let position = entries
        .iter()
        .position(|entry| normalize(&entry.desktop_id).eq_ignore_ascii_case(app_id))
        .or_else(|| {
            entries.iter().position(|entry| {
                entry
                    .startup_wm_class
                    .as_deref()
                    .is_some_and(|class| class.eq_ignore_ascii_case(app_id))
            })
        })
        .or_else(|| {
            entries.iter().position(|entry| {
                last_segment(normalize(&entry.desktop_id))
                    .eq_ignore_ascii_case(last_segment(app_id))
            })
        });

    position.map(|i| &entries[i])
}
