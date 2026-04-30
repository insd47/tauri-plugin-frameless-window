import { invoke } from '@tauri-apps/api/core';

export interface OpenPopupOptions {
  args?: Record<string, string | number | undefined | null>;
  blocking?: boolean;
  detach?: boolean;
}

export async function openPopup(route: string, options?: OpenPopupOptions): Promise<boolean> {
  const { args } = options ?? {};

  if (args) {
    for (const key in args) {
      if (args[key] == null) {
        delete args[key];
      } else if (typeof args[key] === 'number') {
        args[key] = String(args[key]);
      }
    }
  }

  return invoke<boolean>('plugin:frameless-window|open_popup', {
    route,
    args,
    blocking: options?.blocking,
    detach: options?.detach,
  });
}

export async function closePopup(resolved: boolean = false): Promise<void> {
  return invoke('plugin:frameless-window|close_popup', { resolved });
}

export async function isPopupSheet(): Promise<boolean> {
  return invoke<boolean>('plugin:frameless-window|is_popup_sheet');
}

export async function showPopup(): Promise<void> {
  return invoke('plugin:frameless-window|show_popup');
}
