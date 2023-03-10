#![cfg(feature = "egui")]

use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex};
use gpu_allocator::vulkan::Allocator;
use egui_winit_ash_integration::Integration;

pub type GuiHandler = ManuallyDrop<Integration<Arc<Mutex<Allocator>>>>;

pub type GuiEvent = egui::Event;
pub type GuiContext = egui::Context;
pub type Key = egui::Key;
pub use egui;

