const COMMANDS: &[&str] = &["open_popup", "close_popup", "is_popup_sheet", "show_popup"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
