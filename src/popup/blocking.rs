use crate::popup::types::PopupState;
use tauri::{Manager, WebviewWindow, Wry};

pub(crate) fn disable_parent(parent: &WebviewWindow<Wry>, popup: &WebviewWindow<Wry>) {
    platform::disable(parent, popup);
}

pub(crate) fn enable_parent(parent: &WebviewWindow<Wry>) {
    let label = parent.label().to_string();
    let data = parent.state::<PopupState>().blocking_data.clone();
    let entry = data.lock().ok().and_then(|mut locks| locks.remove(&label));

    if let Some(entry) = entry {
        platform::enable(parent, entry);
    }
}

pub(crate) fn is_sheet(window: &WebviewWindow<Wry>) -> bool {
    let state = window.state::<PopupState>();
    state
        .sheet_labels
        .lock()
        .map(|labels| labels.contains(window.label()))
        .unwrap_or(false)
}

pub(crate) fn show_sheet(window: &WebviewWindow<Wry>) {
    platform::show_sheet(window);
}

#[cfg(target_os = "windows")]
mod platform {
    use super::super::types::BlockingData;
    use super::*;
    use raw_window_handle::HasWindowHandle;

    fn get_hwnd(window: &WebviewWindow<Wry>) -> Option<windows_sys::Win32::Foundation::HWND> {
        let handle = window.window_handle().ok()?;
        let raw_window_handle::RawWindowHandle::Win32(win32) = handle.as_ref() else {
            return None;
        };
        Some(win32.hwnd.get() as windows_sys::Win32::Foundation::HWND)
    }

    pub(super) fn disable(parent: &WebviewWindow<Wry>, _popup: &WebviewWindow<Wry>) {
        let Some(hwnd) = get_hwnd(parent) else { return };

        unsafe {
            windows_sys::Win32::UI::Input::KeyboardAndMouse::EnableWindow(hwnd, 0);
        }

        let label = parent.label().to_string();
        let data = parent.state::<PopupState>().blocking_data.clone();
        drop(data.lock().map(|mut locks| {
            locks.insert(label, BlockingData {});
        }));
    }

    pub(super) fn enable(parent: &WebviewWindow<Wry>, _entry: BlockingData) {
        let Some(hwnd) = get_hwnd(parent) else { return };

        unsafe {
            windows_sys::Win32::UI::Input::KeyboardAndMouse::EnableWindow(hwnd, 1);
        }

        let _ = parent.set_focus();
    }

    pub(super) fn show_sheet(_window: &WebviewWindow<Wry>) {}
}

#[cfg(target_os = "macos")]
mod platform {
    use super::super::types::{BlockingData, SendRetained};
    use super::*;
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use raw_window_handle::HasWindowHandle;

    fn get_ns_window(window: &WebviewWindow<Wry>) -> Option<Retained<AnyObject>> {
        let handle = window.window_handle().ok()?;
        let raw_window_handle::RawWindowHandle::AppKit(appkit) = handle.as_ref() else {
            return None;
        };
        let ns_view_ptr = appkit.ns_view.as_ptr() as *const AnyObject;
        unsafe {
            let ns_window: *const AnyObject = objc2::msg_send![&*ns_view_ptr, window];
            Some(Retained::retain(ns_window as *mut AnyObject)?)
        }
    }

    pub(super) fn disable(parent: &WebviewWindow<Wry>, popup: &WebviewWindow<Wry>) {
        let Some(parent_nswindow) = get_ns_window(parent) else {
            return;
        };
        let Some(popup_nswindow) = get_ns_window(popup) else {
            return;
        };

        let popup_label = popup.label().to_string();
        let state = parent.state::<PopupState>();
        if let Ok(mut labels) = state.sheet_labels.lock() {
            labels.insert(popup_label);
        }

        let label = parent.label().to_string();
        let data = state.blocking_data.clone();
        drop(data.lock().map(|mut locks| {
            locks.insert(
                label,
                BlockingData {
                    parent_nswindow: SendRetained(parent_nswindow),
                    popup_nswindow: SendRetained(popup_nswindow),
                },
            );
        }));
    }

    pub(super) fn enable(parent: &WebviewWindow<Wry>, entry: BlockingData) {
        let parent_ptr = Retained::as_ptr(&entry.parent_nswindow.0) as usize;
        let popup_ptr = Retained::as_ptr(&entry.popup_nswindow.0) as usize;

        let (tx, rx) = std::sync::mpsc::channel();

        let _ = parent.run_on_main_thread(move || {
            unsafe {
                let parent_ref = &*(parent_ptr as *const AnyObject);
                let popup_ref = &*(popup_ptr as *const AnyObject);
                let _: () = objc2::msg_send![popup_ref, orderOut: std::ptr::null::<AnyObject>()];
                let _: () = objc2::msg_send![parent_ref, endSheet: popup_ref];
            }
            let _ = tx.send(());
        });

        let _ = rx.recv();
    }

    pub(super) fn show_sheet(window: &WebviewWindow<Wry>) {
        let state = window.state::<PopupState>();
        let data = state.blocking_data.clone();

        let parent_ptr = data.lock().ok().and_then(|locks| {
            for entry in locks.values() {
                let popup_ptr = Retained::as_ptr(&entry.popup_nswindow.0) as usize;
                let this_ptr = get_ns_window(window).map(|w| Retained::as_ptr(&w) as usize);

                if this_ptr == Some(popup_ptr) {
                    return Some(Retained::as_ptr(&entry.parent_nswindow.0) as usize);
                }
            }
            None
        });

        let Some(parent_ptr) = parent_ptr else { return };
        let Some(popup_nswindow) = get_ns_window(window) else {
            return;
        };
        let popup_ptr = Retained::as_ptr(&popup_nswindow) as usize;

        let _ = window.run_on_main_thread(move || {
            let completion_block = block2::RcBlock::new(|_response: isize| {});

            unsafe {
                let parent_ref = &*(parent_ptr as *const AnyObject);
                let popup_ref = &*(popup_ptr as *const AnyObject);
                let _: () = objc2::msg_send![
                    parent_ref,
                    beginSheet: popup_ref,
                    completionHandler: &*completion_block
                ];
            }
        });
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod platform {
    use super::super::types::BlockingData;
    use super::*;

    pub(super) fn disable(parent: &WebviewWindow<Wry>, _popup: &WebviewWindow<Wry>) {
        let label = parent.label().to_string();
        let data = parent.state::<PopupState>().blocking_data.clone();
        drop(data.lock().map(|mut locks| {
            locks.insert(label, BlockingData {});
        }));
    }

    pub(super) fn enable(_parent: &WebviewWindow<Wry>, _entry: BlockingData) {}
    pub(super) fn show_sheet(_window: &WebviewWindow<Wry>) {}
}
