#![cfg(feature = "egui")]

use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex};
use gpu_allocator::vulkan::Allocator;
use egui_winit_ash_integration::Integration;

pub type GuiHandler = ManuallyDrop<Integration<Arc<Mutex<Allocator>>>>;

pub use egui::Context as GuiContext;
pub use egui as gui;
