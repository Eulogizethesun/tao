/// Message to the per-window decor watcher task (see `Window::decor_watch`).
/// Dispatches, decor-change events and rechecks flow through the SAME
/// unbounded channel, so the watcher observes them in causal order.
pub(crate) enum DecorWatchMsg {
  /// A new set_inner_size dispatch. Replaces (supersedes) any active one.
  Dispatch {
    client: openharmony_ability_plugin_window::WindowClient,
    /// Outer width to set (width is never decor-compensated).
    w: i64,
    /// Requested INNER height (physical px) — the correction target.
    req_h: i64,
    /// Outer height dispatched (req_h + decor estimate at dispatch time).
    outer_h: i64,
    /// Window height BEFORE this dispatch (to detect "resize not landed yet").
    pre_h: i64,
    /// Decor estimate used for this dispatch.
    decor_used: i32,
  },
  /// The cached main-window decor changed (app decor_change_callback).
  Decor(i32),
  /// Delayed recheck: our own resize had not landed when the Decor event
  /// arrived (audit P1-B) — re-evaluate with a fresh decor read.
  Recheck,
}

/// Registered handle of a window's decor watcher (one per window, created
/// lazily by the first correctable set_inner_size call).
pub(crate) struct DecorWatchHandle {
  pub(crate) tx: tokio::sync::mpsc::UnboundedSender<DecorWatchMsg>,
  /// Id in openharmony-ability's decor_change_callbacks registry.
  pub(crate) cb_id: u64,
}

/// Fallback recheck budget per dispatch: one Recheck every 500ms while our
/// resize hasn't landed (pathological — normally it lands within tens of ms),
/// giving up after ~30s so a resize that never lands can't spin forever.
const ACTIVE_RESIZE_RECHECKS: u32 = 60;

/// The single in-flight resize a decor watcher is tracking.
struct ActiveResize {
  client: openharmony_ability_plugin_window::WindowClient,
  w: i64,
  req_h: i64,
  outer_h: i64,
  pre_h: i64,
  decor_used: i32,
  rechecks_left: u32,
}

/// Evaluate one decor observation (event or recheck) against the active
/// dispatch. See `run_decor_watch` for the correction/deactivation rules.
async fn process_decor_observation(
  active: &mut Option<ActiveResize>,
  app: &openharmony_ability::OpenHarmonyApp,
  window_id: i64,
  tx: &tokio::sync::mpsc::UnboundedSender<DecorWatchMsg>,
  decor_now: i32,
) {
  let Some(a) = active.as_mut() else { return };
  if decor_now == a.decor_used {
    return; // same estimate — nothing to correct
  }
  if decor_now < a.decor_used {
    // Downward: runtime menubar hide (146 → 66) or equivalent. The inner
    // area grows by itself; correcting would shrink the outer frame. (The
    // content area ends up LARGER than req_h by the decor delta — the
    // expected effect of hiding the menubar.) Treat as "layout intent
    // settled" and drop the active dispatch.
    *active = None;
    return;
  }
  let current = app.window_rect_for(window_id).height as i64;
  if current == a.pre_h {
    // Our own resize has not landed yet — schedule a delayed recheck instead
    // of misreading this as an external change (audit P1-B).
    let tx_retry = tx.clone();
    tokio::spawn(async move {
      tokio::time::sleep(std::time::Duration::from_millis(500)).await;
      let _ = tx_retry.send(DecorWatchMsg::Recheck);
    });
    return;
  }
  if current != a.outer_h {
    // Window moved on without us — don't stomp an external resize.
    *active = None;
    return;
  }
  let corrected = a.req_h.saturating_add(decor_now as i64);
  a.pre_h = a.outer_h;
  a.outer_h = corrected;
  a.decor_used = decor_now;
  a.rechecks_left = ACTIVE_RESIZE_RECHECKS;
  if let Err(e) = a.client.resize_window(window_id, a.w, corrected).await {
    log::warn!("[tao-ohos] resize_window (self-correct) failed for window {}: {:?}", window_id, e);
    *active = None;
  }
}

/// Per-window decor watcher task: consumes Dispatch/Decor/Recheck messages and,
/// while a dispatch is active and the decor estimate GROWS (startup layout
/// convergence — observed 70 → 146 on the reference device), re-dispatches the
/// corrected outer height so the requested INNER height survives.
///
/// Event-driven replacement of the former 15s polling loop — no periodic
/// timer, so arbitrarily slow cold starts (frontend loading for 20-30s) are
/// still corrected. Deactivation rules (stop correcting the active dispatch):
/// - downward decor change (menubar hidden at runtime: the content area GROWS
///   naturally; re-dispatching would wrongly shrink the outer frame);
/// - the window height was changed by anyone else (user drag / another
///   resize source) — never stomp an external resize;
/// - a newer Dispatch (supersession);
/// - recheck budget exhausted while our resize never landed (pathological).
pub(crate) async fn run_decor_watch(
  app: openharmony_ability::OpenHarmonyApp,
  window_id: i64,
  tx: tokio::sync::mpsc::UnboundedSender<DecorWatchMsg>,
  mut rx: tokio::sync::mpsc::UnboundedReceiver<DecorWatchMsg>,
) {
  let mut active: Option<ActiveResize> = None;
  loop {
    let Some(msg) = rx.recv().await else { break }; // window dropped — exit
    match msg {
      DecorWatchMsg::Dispatch { client, w, req_h, outer_h, pre_h, decor_used } => {
        if let Err(e) = client.resize_window(window_id, w, outer_h).await {
          log::warn!("[tao-ohos] resize_window failed for window {}: {:?}", window_id, e);
        }
        // A no-op resize (target == current height) has nothing to "land":
        // sentinel pre_h so the not-landed-yet guard never matches and the
        // first Decor event corrects normally (audit P1-A).
        let pre_h = if outer_h == pre_h { i64::MIN } else { pre_h };
        active = Some(ActiveResize {
          client,
          w,
          req_h,
          outer_h,
          pre_h,
          decor_used,
          rechecks_left: ACTIVE_RESIZE_RECHECKS,
        });
      }
      DecorWatchMsg::Decor(decor_now) => {
        process_decor_observation(&mut active, &app, window_id, &tx, decor_now).await;
      }
      DecorWatchMsg::Recheck => {
        if active.as_ref().is_some_and(|a| a.rechecks_left == 0) {
          // Our resize still hasn't landed after the full budget — give up.
          active = None;
          continue;
        }
        if let Some(a) = active.as_mut() {
          a.rechecks_left -= 1;
        }
        // decor_height_for (issue #87 major-10): window-keyed decor lookup —
        // equivalent to decor_height() here because the watcher is only
        // installed for UIAbility windows (Float returns before
        // ensure_decor_watch), but stays correct if that ever changes.
        let decor_now = app.decor_height_for(window_id);
        process_decor_observation(&mut active, &app, window_id, &tx, decor_now).await;
      }
    }
  }
}
