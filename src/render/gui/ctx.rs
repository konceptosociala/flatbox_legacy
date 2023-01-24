//~ use std::sync::Arc;
//~ use egui_winit_ash_integration::*;
//~ use egui::*;
//~ use egui_winit::*;
//~ use ash::vk;

//~ use crate::render::renderer::Renderer;

//~ pub trait EguiExt {
	//~ type InitError;
	
	//~ fn init(renderer: &Renderer) -> Result<Self, Self::InitError>
	//~ where 
		//~ Self: Sized;
//~ }

//~ impl EguiExt for Context {
	//~ type InitError = vk::Result;
	
	//~ fn init(renderer: &Renderer) -> Result<Self, Self::InitError> {
		
				
		//~ Ok(integration.context())
	//~ }
//~ }
