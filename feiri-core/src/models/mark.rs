use crate::models::Window;
use serde::{Deserialize, Serialize};

/// A numbered mark associated with a window.
///
/// Marks allow a window to be referenced later by its slot.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Mark {
    /// Numeric slot assigned to the mark. Currently the mark store
    /// only supports slots `1` through `9`. Any slot usage beyond `9`
    /// will cause the program to crash.
    ///
    /// For example: `1`, `2`, or `3`.
    pub slot: usize,

    /// Feiri window currently associated with the mark
    pub window: Window,
}
