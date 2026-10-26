// Copyright 2014-2021 The winit contributors
// Copyright 2021-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0

//! Contains traits with platform-specific methods in them.
//!
//! Contains the follow OS-specific modules:
//!
//!  - `android`
//!  - `ios`
//!  - `macos`
//!  - `unix`
//!  - `linux`
//!  - `windows`
//!
//! And the following platform-specific module:
//!
//! - `run_return` (available on `windows`, `unix`, `macos`, and `android`)
//!
//! However only the module corresponding to the platform you're compiling to will be available.

/// Contains items that are specific to the Android platform.
pub mod android;
/// Contains items that are specific to the iOS platform.
pub mod ios;
/// Contains items that are specific to the Linux platform.
pub mod linux;
/// Contains items that are specific to the macOS platform.
pub mod macos;
/// Contains items that are specific to the OpenHarmony platform.
pub mod ohos;
/// Contains the `run_return` extension shared by several platforms.
pub mod run_return;
/// Contains items that are specific to the Unix platforms.
pub mod unix;
/// Contains items that are specific to the Windows platform.
pub mod windows;
