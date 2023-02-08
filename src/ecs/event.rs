use std::{
	sync::{
		mpsc::TryRecvError,
		Arc,
	},
	any::{TypeId, Any},
	fmt::Debug,
};

use thiserror::Error;
use bus::*;

/// Error for handling events
#[derive(Error, Debug)]
pub enum EventError {
	#[error("Couldn't broadcast an event")]
	Send(Arc<(dyn Any + Send + Sync)>),
	#[error("Couldn't read an event")]
	Read(#[from] TryRecvError),
}

/// Broadcasts events, which are sent to it
pub struct EventWriter {
	writer: Bus<Arc<dyn Any + Send + Sync>>,
}

impl EventWriter {
	/// Create new instance of [`EventWriter`]
	pub fn new() -> Self {
		Self { 
			writer: Bus::new(200),
		}
	}
	
	/// Send event of type which implements [`Event`] trait
	pub fn send(&mut self, event: Arc<(dyn Any + Send + Sync)>) -> Result<(), EventError> {		
		match self.writer.try_broadcast(event){
			Ok(()) => Ok(()),
			Err(ev) => Err(EventError::Send(ev)),
		}
	}
}

/// Reads events from [`EventWriter`]. It is unique for each system (for spmc implementation)
pub struct EventReader {
	reader: BusReader<Arc<(dyn Any + Send + Sync)>>,
}

impl EventReader {
	/// Create new instance of [`EventReader`] and add it to [`EventWriter`]
	pub fn new(event_writer: &mut EventWriter) -> Self {
		Self { 
			reader: event_writer.writer.add_rx(),
		}
	}
	
	/// Send event of type `E`, which implements [`Event`] trait
	pub fn read<E: Any + Clone + Send + Sync + 'static>(&mut self) -> Result<E, EventError> {
		match self.reader.try_recv(){
			Ok(ev) => if (*ev).type_id() == TypeId::of::<E>() {
				Ok((*ev.downcast::<E>().unwrap()).clone())
			} else {
				Err(EventError::Read(TryRecvError::Empty))
			},
			Err(err) => Err(EventError::Read(err))
		}
	}
}
