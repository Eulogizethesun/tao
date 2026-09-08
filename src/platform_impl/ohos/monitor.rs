use crate::dpi::{PhysicalPosition, PhysicalSize};
use crate::monitor;
use openharmony_ability::OpenHarmonyApp;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MonitorHandle {
  app: OpenHarmonyApp,
}

impl MonitorHandle {
  pub(crate) fn new(app: OpenHarmonyApp) -> Self {
    Self { app }
  }

  pub fn name(&self) -> Option<String> {
    Some("OpenHarmony Device".to_owned())
  }

  pub fn size(&self) -> PhysicalSize<u32> {
    // Real physical display dimensions — NOT the window's content_rect (which is
    // the window's own content area and is smaller than the screen). Using
    // content_rect here made positioner `Center` compute to negative coords
    // (content/2 - outer/2 < 0) which OHOS clamps to (0,0), so windows snapped
    // to top-left instead of centering.
    // Prefer OHOS DisplayManager physical pixels; fall back to content_rect
    // when the query returns 0. See ohos-monitor-real-values.
    let w = self.app.display_width();
    let h = self.app.display_height();
    if w > 0 && h > 0 {
      PhysicalSize::new(w, h)
    } else {
      warn!("[tao ohos] DisplayManager size query returned 0; falling back to content_rect");
      let size = self.app.content_rect();
      PhysicalSize::new(size.width as _, size.height as _)
    }
  }

  pub fn position(&self) -> PhysicalPosition<i32> {
    (0, 0).into()
  }

  pub fn scale_factor(&self) -> f64 {
    self.app.scale() as f64
  }

  pub fn video_modes(&self) -> impl Iterator<Item = monitor::VideoMode> {
    let size = self.size().into();
    // refresh_rate from OHOS DisplayManager real value (see ohos-monitor-real-values).
    // bit_depth fixed at 32 (RGBA8888) — see ohos-monitor-degradation.
    std::iter::once(monitor::VideoMode {
      video_mode: VideoMode {
        size,
        bit_depth: 32,
        refresh_rate: self.app.refresh_rate() as u16,
        monitor: self.clone(),
      },
    })
  }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct VideoMode {
  size: (u32, u32),
  bit_depth: u16,
  refresh_rate: u16,
  monitor: MonitorHandle,
}

impl VideoMode {
  pub fn size(&self) -> PhysicalSize<u32> {
    self.size.into()
  }

  pub fn bit_depth(&self) -> u16 {
    self.bit_depth
  }

  pub fn refresh_rate(&self) -> u16 {
    self.refresh_rate
  }

  pub fn monitor(&self) -> monitor::MonitorHandle {
    monitor::MonitorHandle {
      inner: self.monitor.clone(),
    }
  }
}
