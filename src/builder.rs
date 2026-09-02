use tauri::window::{Effect, EffectsBuilder};
use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder, Wry};

pub trait WebviewWindowBuilderExt<R: Runtime>: Sized {
    fn frameless<L: Into<String>>(
        manager: &'_ AppHandle<R>,
        label: L,
        url: WebviewUrl,
    ) -> WebviewWindowBuilder<'_, Wry, AppHandle<R>>
    where
        AppHandle<R>: Manager<Wry>;

    fn effect(self, effect: Effect) -> Self;
}

impl<R: Runtime> WebviewWindowBuilderExt<R> for WebviewWindowBuilder<'_, R, AppHandle<R>> {
    fn frameless<L: Into<String>>(
        manager: &'_ AppHandle<R>,
        label: L,
        url: WebviewUrl,
    ) -> WebviewWindowBuilder<'_, Wry, AppHandle<R>>
    where
        AppHandle<R>: Manager<Wry>,
    {
        let label = label.into();
        let builder = WebviewWindowBuilder::new(manager, label, url).visible(false);

        #[cfg(target_os = "macos")]
        let builder = {
            use tauri::TitleBarStyle;
            builder.title_bar_style(TitleBarStyle::Overlay).hidden_title(true)
        };

        #[cfg(target_os = "windows")]
        let builder = {
            use tauri::webview::ScrollBarStyle::FluentOverlay;
            use tauri_plugin_window_controls::WindowControlsBuilderExt;
            builder.window_controls_height(32).scroll_bar_style(FluentOverlay)
        };

        builder
    }

    fn effect(self, effect: Effect) -> Self {
        self.effects(EffectsBuilder::new().effect(effect).build())
    }
}
