#![allow(unused_imports)]
use std::{
	sync::{
		mpsc::TryRecvError,
		Arc,
	},
	any::TypeId,
	collections::HashMap,
};

use thiserror::Error;
use bus::*;
use winit::event::*;

use crate::Despero;

/// Error for handling events
#[derive(Error, Debug)]
pub enum EventError {
	#[error("Couldn't broadcast an event")]
	Send(HashMap<TypeId, Arc<(dyn Event + Send + Sync)>>),
	#[error("Couldn't read an event")]
	Read(#[from] TryRecvError),
}

/// Universal trait for all events
pub trait Event: std::fmt::Debug {}

/// Implement [`Event`] for [`KeyboardInput`]
impl Event for KeyboardInput {}

/// Broadcasts events, which are sent to it. Part of [`Despero`] struct
pub struct EventWriter {
	writer: Bus<HashMap<TypeId, Arc<dyn Event + Send + Sync>>>,
}

impl EventWriter {
	/// Create new instance of [`EventWriter`]
	pub(crate) fn new() -> Self {
		Self { 
			writer: Bus::new(100),
		}
	}
	
	/// Send event of type `E`, which implements [`Event`] trait
	pub fn send<E>(&mut self, event: Arc<(dyn Event + Send + Sync)>) -> Result<(), EventError>
	where
		E: Event + Sync + 'static,
    {
		let mut events = HashMap::new();
		let typeid = TypeId::of::<E>();
		events.insert(typeid, event);
		
		match self.writer.try_broadcast(events.clone()){
			Ok(()) => Ok(()),
			Err(ev) => Err(EventError::Send(ev)),
		}
	}
}

/// Reads events from [`EventWriter`]. It is unique for each system (for spmc implementation)
pub struct EventReader {
	reader: BusReader<HashMap<TypeId, Arc<(dyn Event + Send + Sync)>>>,
}

impl EventReader {
	/// Create new instance of [`EventReader`] and add it to [`EventWriter`]
	pub fn new(event_writer: &mut EventWriter) -> Self {
		Self { 
			reader: event_writer.writer.add_rx(),
		}
	}
	
	/// Send event of type `E`, which implements [`Event`] trait
	pub fn read<E: Event + Sync + 'static>(&mut self) -> Result<Arc<(dyn Event + Send + Sync)>, EventError> {
		match self.reader.try_recv(){
			Ok(ev) => match ev.get(&TypeId::of::<E>()) {
				Some(ev) => Ok(ev.clone()),
				_ => Err(EventError::Read(TryRecvError::Empty)),
			},
			Err(err) => Err(EventError::Read(err))
		}
	}
}
