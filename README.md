# tauri-plugin-frameless-window

Frameless window presets and route-based popup windows for Tauri v2.

This plugin provides the frameless window preset and the popup lifecycle. On Windows, it configures native caption
controls through [`tauri-plugin-window-controls`](https://github.com/insd47/tauri-plugin-window-controls).

## Features

- **Frameless window builder**: `WebviewWindowBuilder::frameless(...)` creates a window with the platform-specific
  frameless preset.
- **Single window effect shortcut**: `.effect(Effect::Mica)` and `window.set_effect(Effect::Mica)` wrap Tauri's
  `EffectsBuilder` ceremony for one effect.
- **Windows caption controls**: Windows frameless windows use native minimize, maximize, and close controls with
  Windows 11 Snap Layout support.
- **Route-based popups**: `openPopup('error/crash', { args: { message } })` opens `/popup/error/crash?message=...`.
- **Blocking popup mode**: `blocking: true` presents a native sheet on macOS and disables the parent window on Windows.
- **Promise resolution**: `closePopup(true | false)` from inside the popup resolves the caller's `openPopup(...)`
  promise.

## Install

From your Tauri app's `src-tauri` directory, install the Rust plugins:

```bash
cargo add tauri-plugin-frameless-window
cargo add tauri-plugin-window-controls@0.2.1
```

From your frontend package, install the JavaScript package if you use the popup helpers:

```bash
pnpm add tauri-plugin-frameless-window
```

`tauri-plugin-window-controls` must be a direct Rust dependency of the app because the app registers it explicitly.
Register it before `tauri-plugin-frameless-window`:

```rust
tauri::Builder::default ()
.plugin(tauri_plugin_window_controls::init())
.plugin(tauri_plugin_frameless_window::init())
.run(tauri::generate_context!())
.expect("error while running tauri application");
```

On Windows, configure the application manifest required by `tauri-plugin-window-controls` for layered child windows.
See its [Windows manifest instructions](https://github.com/insd47/tauri-plugin-window-controls#windows-manifest).

Add the default permission to your capability file, for example `src-tauri/capabilities/default.json`:

```json
{
  "permissions": [
    "frameless-window:default"
  ]
}
```

`tauri-plugin-window-controls` does not require a capability permission.

## Frameless Windows

Use `WebviewWindowBuilderExt` when creating a Tauri webview window:

```rust
use tauri::{window::Effect, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_frameless_window::WebviewWindowBuilderExt;

let window = WebviewWindowBuilder::frameless(app, "main", WebviewUrl::App("/".into()))
.title("My App")
.inner_size(1280.0, 800.0)
.effect(Effect::Mica)
.build() ?;
```

For existing windows, use `WebviewWindowExt` when applying an effect after creation:

```rust
use tauri::window::Effect;
use tauri_plugin_frameless_window::WebviewWindowExt;

window.set_effect(Effect::Mica) ?;
```

If you need window size or position persistence, use Tauri's official `tauri-plugin-window-state` alongside this plugin.

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
| Windows  | Adds native caption controls with a default height of 32 logical pixels and preserves the native shadow.                                 |
| Linux    | Keeps the default Tauri window shape and only applies `visible(false)`.                                                                   |

Call `.window_controls_height(...)` after `frameless(...)` to override the default Windows control height. The final
call takes precedence.

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
- `WebviewWindowExt`
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
