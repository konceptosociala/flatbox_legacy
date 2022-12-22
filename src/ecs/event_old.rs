use std::sync::Arc;
use std::any::TypeId;
use std::collections::HashMap;
use std::ops::Deref;
use event_listener as el;

/// Trait used for events pushed in [`EventHandler`]
pub trait Event {}

/// Implmenet [`Event`] for KeyboardInput
impl Event for winit::event::KeyboardInput {}

#[readonly::make]
pub struct EventHandler {
	events: HashMap<TypeId, Box<dyn EventList<dyn Event>>>,
}

impl EventHandler {
	pub(crate) fn new() -> Self {
		EventHandler {
			events: HashMap::new(),
		}
	}
	
	pub(crate) fn send<E>(&mut self, event: E)
		where E: Event + 'static
	{
		if let Some(ev) = self.events.get_mut(&TypeId::of::<E>()) {
            ev.push(&event);
            ev.notify();
        }
	}
	
	pub fn get<E: Event + 'static>(&self) -> Option<&(dyn Event + 'static)>  {
		self.events.get(&TypeId::of::<E>())
			.unwrap()
			.deref()
			.handle()
	}
}

#[readonly::make]
pub struct Events<E> {
	#[readonly]
	pub handle: Option<E>,
	event: Arc<el::Event>,
	listener: el::EventListener,
}

impl<E> Default for Events<E> {
	fn default() -> Self {
		let event = Arc::new(el::Event::new());
		Events {
			handle: None,
			event: event.clone(),
			listener: event.listen(),
		}
	}
}

pub trait EventList<E: Event + ?Sized> {	
	fn push(&mut self, event: &E);	
	fn handle(&self) -> Option<&E>;
	fn notify(&mut self);
}

impl<E: Event + Clone + ?Sized> EventList<E> for Events<E> {	
	fn push(&mut self, event: &E){
		self.handle = Some(event.clone());
	}
	
	fn handle(&self) -> Option<&E> {
		self.handle.as_ref()
	}
	
	fn notify(&mut self) {
		self.event.notify(1);
	}
}

unsafe impl Sync for EventHandler {} 
unsafe impl Send for EventHandler {}
