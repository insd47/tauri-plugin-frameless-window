# tauri-plugin-frameless-window

Frameless webview windows for Tauri v2 — overlay title bar on macOS, decoration-less window with shadow on Windows, plus an optional modal popup flow that resolves a Promise from inside the popup.

## Features

- **`WebviewWindowBuilderExt::frameless`** — drop-in builder helper that hides the title bar (overlay on macOS, no decoration + shadow on Windows) and starts hidden so the frontend can `show()` once mounted.
- **Route-based popup** — `openPopup('error/crash', { args: { message } })` opens `/popup/error/crash?message=...` in a frameless popup window.
- **Modal mode** — `blocking: true` makes the popup block the parent: native sheet on macOS, disabled parent on Windows.
- **Resolve-with-promise** — `closePopup(true|false)` from inside the popup resolves the caller's Promise.

## Install

```bash
cargo add tauri-plugin-frameless-window
pnpm add tauri-plugin-frameless-window
```

Register the plugin and allow the default capability:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_frameless_window::init())
    // ...
```

```json
{ "permissions": ["frameless-window:default"] }
```

## Rust — frameless window builder

```rust
use tauri::{WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_frameless_window::WebviewWindowBuilderExt;

let win = WebviewWindowBuilder::frameless(app, "main", WebviewUrl::App("/".into()))
    .title("My App")
    .inner_size(1280.0, 800.0)
    .build()?;
```

`frameless` always starts the window hidden (`visible(false)`); call `window.show()` from the frontend once the UI is ready to avoid a flash of unmounted content.

## TS — popup flow

```tsx
import { openPopup, closePopup, showPopup } from 'tauri-plugin-frameless-window';

// from a parent route
const ok = await openPopup('error/crash', {
  args: { message: 'Something broke' },
  blocking: true,
});

// from inside the /popup/<route> view
await showPopup();      // reveal once mounted (mirrors frameless `visible(false)`)
await closePopup(true); // resolves the caller's promise with `true`
```

The popup's frontend route is whatever `/popup/<route>` resolves to in your app's router.

## Platform notes

| Platform | Frameless | Modal popup |
|---|---|---|
| macOS | Overlay title bar, hidden title | Native NSWindow sheet via `beginSheet:completionHandler:` |
| Windows | No decoration, with shadow | Parent disabled via `EnableWindow(hwnd, FALSE)` |
| Linux | Plain window | Non-modal popup window only |

## License

MIT
