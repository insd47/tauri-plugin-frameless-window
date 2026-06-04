# tauri-plugin-frameless-window

Frameless window presets and route-based popup windows for Tauri v2.

This plugin provides the frameless window preset and the popup lifecycle. On Windows, it delegates the caption-button
overlay to [`tauri-plugin-window-controls`](https://github.com/insd47/tauri-plugin-window-controls), which provides
native-like minimize, maximize, and close controls and applies the required decoration-less window shape with shadow.

## Features

- **Frameless window builder**: `WebviewWindowBuilder::frameless(...)` creates a window with the platform-specific
  frameless preset.
- **Windows caption controls**: Windows frameless windows use `tauri-plugin-window-controls` for native-like caption
  buttons and Windows 11 snap-layout support.
- **Route-based popups**: `openPopup('error/crash', { args: { message } })` opens `/popup/error/crash?message=...`.
- **Blocking popup mode**: `blocking: true` presents a native sheet on macOS and disables the parent window on Windows.
- **Promise resolution**: `closePopup(true | false)` from inside the popup resolves the caller's `openPopup(...)`
  promise.

## Install

From your Tauri app's `src-tauri` directory, install both Rust plugins:

```bash
cargo add tauri-plugin-frameless-window
cargo add tauri-plugin-window-controls
```

From your frontend package, install the JavaScript package if you use the popup helpers:

```bash
pnpm add tauri-plugin-frameless-window
```

`tauri-plugin-window-controls` must be a direct Rust dependency of the app because the app registers it explicitly. Add
it unconditionally, not under a Windows-only target dependency; the plugin is a no-op on non-Windows targets.

Register `tauri-plugin-window-controls` before `tauri-plugin-frameless-window`:

```rust
tauri::Builder::default ()
.plugin(tauri_plugin_window_controls::init())
.plugin(tauri_plugin_frameless_window::init())
.run(tauri::generate_context!())
.expect("error while running tauri application");
```

Add both default permissions to your capability file, for example `src-tauri/capabilities/default.json`:

```json
{
  "permissions": [
    "window-controls:default",
    "frameless-window:default"
  ]
}
```

## Frameless Windows

Use `WebviewWindowBuilderExt` when creating a Tauri webview window:

```rust
use tauri::{WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_frameless_window::WebviewWindowBuilderExt;

let window = WebviewWindowBuilder::frameless(app, "main", WebviewUrl::App("/".into()))
.title("My App")
.inner_size(1280.0, 800.0)
.build() ?;
```

`frameless` always starts the window hidden with `visible(false)`. Call `show()` from the frontend after your UI has
mounted to avoid a flash of unmounted content.

```ts
import {getCurrentWindow} from '@tauri-apps/api/window';

await getCurrentWindow().show();
```

Platform behavior:

| Platform | `frameless` preset                                                                                                                        |
|----------|-------------------------------------------------------------------------------------------------------------------------------------------|
| macOS    | Uses an overlay title bar and hides the native title.                                                                                     |
| Windows  | Enables the `tauri-plugin-window-controls` title-bar overlay. The window-controls plugin applies `decorations(false)` and `shadow(true)`. |
| Linux    | Keeps the default Tauri window shape and only applies `visible(false)`.                                                                   |

## Popup Flow

From a parent window, open a route-based popup:

```ts
import {openPopup} from 'tauri-plugin-frameless-window';

const confirmed = await openPopup('error/crash', {
  args: {message: 'Something broke'},
  blocking: true,
});
```

This opens your app route at `/popup/error/crash?message=Something%20broke`. The route must be handled by your frontend
router.

From inside the popup route, reveal the popup after mount and resolve the parent promise when the user completes the
flow:

```ts
import {closePopup, showPopup} from 'tauri-plugin-frameless-window';

await showPopup();
await closePopup(true);
```

`openPopup(...)` resolves to the boolean passed to `closePopup(...)`.

Popup options:

| Option     | Type                                                    | Description                                                                               |
|------------|---------------------------------------------------------|-------------------------------------------------------------------------------------------|
| `args`     | `Record<string, string \| number \| null \| undefined>` | Query parameters appended to `/popup/<route>`. `null` and `undefined` values are omitted. |
| `blocking` | `boolean`                                               | Blocks the parent on supported platforms.                                                 |
| `detach`   | `boolean`                                               | Opens the popup without assigning the current window as its parent.                       |

Popup platform behavior:

| Platform | `blocking: true` behavior                                                             |
|----------|---------------------------------------------------------------------------------------|
| macOS    | Presents the popup as a native `NSWindow` sheet when `showPopup()` is called.         |
| Windows  | Disables the parent window with `EnableWindow(hwnd, FALSE)` until the popup resolves. |
| Linux    | Opens a non-modal popup window.                                                       |

## Rust Popup API

You can also open popups from Rust with `PopupExt`:

```rust
use tauri::Manager;
use tauri_plugin_frameless_window::PopupExt;

let parent = app.get_webview_window("main").expect("main window missing");

let confirmed = app
.popup("error/crash")
.args([("message", "Something broke")])
.parent(parent)
.blocking()
.open()
.await?;
```

Use `open_detached()` when you need a `PopupHandle` and want to wait manually:

```rust
let handle = app.popup("settings/about").open_detached() ?;
let resolved = handle.wait().await;
```

## API

Rust exports:

- `init()`
- `WebviewWindowBuilderExt`
- `PopupExt`
- `PopupBuilder`
- `PopupHandle`
- `is_popup_window_label(label)`

JavaScript exports:

- `openPopup(route, options?)`
- `closePopup(resolved?)`
- `showPopup()`
- `isPopupSheet()`

## License

MIT
