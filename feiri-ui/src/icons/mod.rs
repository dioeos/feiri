mod discover;
pub mod entry;
mod error;

use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::{Component, Path, PathBuf},
};

use entry::DesktopEntry;

type Section = HashMap<String, String>;
type ThemeIndex = HashMap<String, Section>;

pub fn resolve_icon(entry: &DesktopEntry, theme: &str) -> Option<PathBuf> {
    let icon = entry.icon()?;
    let path = Path::new(icon);
    if path.is_absolute() {
        return path.is_file().then(|| path.to_path_buf());
    }
    if !valid_name(icon) {
        return None;
    }

    let data_dirs = discover::data_dirs();
    let mut roots = Vec::new();
    if let Some(home) = env::var_os("HOME").filter(|s| !s.is_empty()) {
        roots.push(PathBuf::from(home).join(".icons"));
    }
    roots.extend(data_dirs.iter().map(|dir| dir.join("icons")));

    let theme = if theme.is_empty() { "hicolor" } else { theme };
    lookup(&roots, theme, icon, 32).or_else(|| {
        roots
            .iter()
            .cloned()
            .chain(data_dirs.iter().map(|dir| dir.join("pixmaps")))
            .find_map(|dir| image_in(&dir, icon))
    })
}

fn valid_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains('/')
}

fn read_theme_index(path: &Path) -> Option<ThemeIndex> {
    let text = fs::read_to_string(path).ok()?;
    let mut sections = ThemeIndex::new();
    let mut section = String::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            section = name.to_owned();
        } else if let Some((key, value)) = line.split_once('=') {
            sections
                .entry(section.clone())
                .or_default()
                .insert(key.trim().to_owned(), value.trim().to_owned());
        }
    }
    Some(sections)
}

fn size_distance(section: &Section, requested: u32) -> Option<u32> {
    let number = |key: &str, default: u32| {
        section
            .get(key)
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(default)
    };
    let size = section.get("Size")?.parse::<u32>().ok()?;
    let scale = number("Scale", 1);
    let (min, max) = match section.get("Type").map(String::as_str) {
        Some("Fixed") => (size, size),
        Some("Scalable") => (number("MinSize", size), number("MaxSize", size)),
        _ => {
            let threshold = number("Threshold", 2);
            (
                size.saturating_sub(threshold),
                size.saturating_add(threshold),
            )
        }
    };
    if scale == 0 || min > max {
        return None;
    }
    let min = min.saturating_mul(scale);
    let max = max.saturating_mul(scale);
    Some(if requested < min {
        min - requested
    } else {
        requested.saturating_sub(max)
    })
}

fn image_in(directory: &Path, icon: &str) -> Option<PathBuf> {
    ["png", "svg", "xpm"]
        .into_iter()
        .map(|extension| directory.join(format!("{icon}.{extension}")))
        .find(|path| path.is_file())
}

fn lookup(roots: &[PathBuf], theme: &str, icon: &str, size: u32) -> Option<PathBuf> {
    let mut visited = HashSet::new();
    search_theme(roots, theme, icon, size, &mut visited)
        .or_else(|| search_theme(roots, "hicolor", icon, size, &mut visited))
}

fn search_theme(
    roots: &[PathBuf],
    theme: &str,
    icon: &str,
    size: u32,
    visited: &mut HashSet<String>,
) -> Option<PathBuf> {
    if !valid_name(theme) || !visited.insert(theme.to_owned()) {
        return None;
    }
    let index = roots
        .iter()
        .find_map(|root| read_theme_index(&root.join(theme).join("index.theme")))?;
    let metadata = index.get("Icon Theme")?;
    let mut best: Option<(u32, PathBuf)> = None;

    for key in ["Directories", "ScaledDirectories"] {
        for directory in metadata
            .get(key)
            .into_iter()
            .flat_map(|s| s.split(','))
            .map(str::trim)
        {
            if directory.is_empty()
                || !Path::new(directory)
                    .components()
                    .all(|c| matches!(c, Component::Normal(_)))
            {
                continue;
            }
            let Some(distance) = index.get(directory).and_then(|s| size_distance(s, size)) else {
                continue;
            };
            for root in roots {
                if let Some(path) = image_in(&root.join(theme).join(directory), icon) {
                    if distance == 0 {
                        return Some(path);
                    }
                    if best.as_ref().is_none_or(|(d, _)| distance < *d) {
                        best = Some((distance, path));
                    }
                }
            }
        }
    }
    if let Some((_, path)) = best {
        return Some(path);
    }
    for parent in metadata
        .get("Inherits")
        .into_iter()
        .flat_map(|s| s.split(','))
        .map(str::trim)
    {
        if let Some(path) = search_theme(roots, parent, icon, size, visited) {
            return Some(path);
        }
    }
    None
}
