#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;
use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder, Wry};

pub trait WebviewWindowBuilderExt<R: Runtime> {
    fn frameless<L: Into<String>>(
        manager: &'_ AppHandle<R>,
        label: L,
        url: WebviewUrl,
    ) -> WebviewWindowBuilder<'_, Wry, AppHandle<R>>
    where
        AppHandle<R>: Manager<Wry>;
}

impl<R: Runtime> WebviewWindowBuilderExt<R> for WebviewWindowBuilder<'_, R, AppHandle<R>>
where
    AppHandle<R>: Manager<Wry>,
{
    fn frameless<L: Into<String>>(
        manager: &'_ AppHandle<R>,
        label: L,
        url: WebviewUrl,
    ) -> WebviewWindowBuilder<'_, Wry, AppHandle<R>> {
        let label = label.into();
        let builder = WebviewWindowBuilder::new(manager, label, url).visible(false);

        #[cfg(target_os = "macos")]
        let builder = builder
            .title_bar_style(TitleBarStyle::Overlay)
            .hidden_title(true);

        #[cfg(target_os = "windows")]
        let builder = builder.decorations(false).shadow(true);

        builder
    }
}
