#![allow(unused_imports)]
use std::sync::mpsc::TryRecvError;
use std::any::TypeId;
use std::collections::HashMap;
use std::ops::Deref;
use winit::event::KeyboardInput;
use thiserror::Error;
use hecs_schedule::System;
use bus::*;
use crate::Despero;

/// Universal trait for all events
pub trait Event {}

/// Implement [`Event`] for [`winit::event::KeyboardInput`]
impl Event for KeyboardInput {}

/// Error for handling events
#[derive(Error, Debug)]
pub enum EventError<E> {
	#[error("Couldn't broadcast an event {0}")]
	Send(E),
	#[error("Couldn't read an event")]
	Read(#[from] TryRecvError),
}

/// Broadcasts events, which are sent to it. Part of [`Despero`] struct
pub struct EventWriter<E: Clone + Sync> {
	writer: Bus<E>,
}

impl<E: Clone + Sync> EventWriter<E> {
	pub(crate) fn new() -> Self {
		Self { 
			writer: Bus::new(100),
		}
	}
	
	pub fn send(&mut self, event: E) -> Result<(), EventError<E>> {
		match self.writer.try_broadcast(event) {
			Ok(()) => Ok(()),
			Err(ev) => Err(EventError::Send(ev))
		}
	}
}

/// Reads events from [`EventWriter`]. It is unique for each system (for spmc implementation)
pub struct EventReader<E: Clone + Sync> {
	reader: BusReader<E>,
}

impl<E: Clone + Sync> EventReader<E> {
	pub fn new(event_writer: &mut EventWriter<E>) -> Self {
		Self { 
			reader: event_writer.writer.add_rx(),
		}
	}
	
	pub fn read(&mut self) -> Result<E, EventError<E>> {
		match self.reader.try_recv() {
			Ok(ev) => Ok(ev),
			Err(err) => Err(EventError::Read(err))
		}
	}
}

/// Trait for adding [`EventReader`] to context
pub trait AddEventReader<E: Clone + Sync> {
	fn add_reader(&mut self, event_reader: EventReader<E>);
}

impl<E: Clone + Sync, F: FnOnce()> AddEventReader<E> for F {
	fn add_reader(&mut self, _event_reader: EventReader<E>){
		
	}
}
