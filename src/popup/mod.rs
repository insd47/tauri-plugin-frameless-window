mod blocking;
pub(crate) mod builder;
pub(crate) mod types;

use tauri::{Manager, Wry};
use uuid::Uuid;

pub use builder::PopupBuilder;
pub use types::{PopupExt, PopupHandle};

pub(crate) use blocking::{is_sheet, show_sheet};
pub(crate) use types::PopupState;

const WINDOW_LABEL_PREFIX: &str = "popup";
pub(crate) const POPUP_ROUTE_PREFIX: &str = "/popup";

pub(crate) fn init<M: Manager<Wry>>(manager: &M) {
    manager.manage(PopupState::default());
}

pub fn is_popup_window_label(label: &str) -> bool {
    label.starts_with(WINDOW_LABEL_PREFIX)
}

pub(crate) fn popup_window_label() -> String {
    format!("{WINDOW_LABEL_PREFIX}-{}", Uuid::new_v4())
}
