pub use anyhow::{Error, Result};
pub use wgpu;
pub use winit;

mod app_container;
mod wgpu_base;

pub use app_container::WgpuApplication;
pub use app_container::run;
pub use wgpu_base::WgpuBase;

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
