// OHOS backend. Formerly a single ~3200-line mod.rs; split by concern
// (tao issue #84, item 6), with this module kept as a re-export facade so all
// `crate::platform_impl::ohos::*` / `crate::platform_impl::*` paths are
// unchanged:
//
// - `event_loop`: EventLoop/Proxy/WindowTarget, input-event translation,
//   run_loop dispatch, theme/cursor/monitor shared helpers.
// - `window`: Window (builder attrs → setters → drop), WindowId, the
//   windowStatusChange state mirror registry, scancode stubs.
// - `decor_watch`: the per-window set_inner_size decor self-correction task.
// - `monitor`: MonitorHandle/VideoMode (single-display device).
// - `keycodes`: OHOS keycode → tao Key/KeyLocation mapping.

mod decor_watch;
mod event_loop;
mod keycodes;
mod monitor;
mod window;

pub use event_loop::{
  DeviceId, EventLoop, EventLoopProxy, EventLoopWindowTarget, KeyEventExtra,
};
pub use monitor::{MonitorHandle, VideoMode};
pub use window::{
  keycode_from_scancode, keycode_to_scancode, OHOSWindowKind, OsError,
  PlatformSpecificWindowBuilderAttributes,
};

pub(crate) use event_loop::PlatformSpecificEventLoopAttributes;
pub(crate) use window::{Window, WindowId};

pub(crate) use crate::icon::NoIcon as PlatformIcon;
