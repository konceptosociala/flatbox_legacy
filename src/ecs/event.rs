use std::sync::Arc;
use event_listener as el;

#[readonly::make]
pub struct Events<H>
	where H: Event
{
	#[readonly]
	pub handle: Option<H>,
	event: Arc<el::Event>,
	listener: el::EventListener,
}

#[readonly::make]
pub struct EventHandler {
    pub events: Vec<Box<dyn EventList>>,
}

impl EventHandler {
	pub(crate) fn new() -> Self {
		EventHandler {
			events: vec![],
		}
	}
	
	pub(crate) fn send<E>(&self, event: E)
		where E: Event
	{
		todo!();
	}
}

impl<H> Default for Events<H>
	where H: Event
{
	fn default() -> Self {
		let event = Arc::new(el::Event::new());
		Events {
			handle: None,
			event: event.clone(),
			listener: event.listen(),
		}
	}
}

pub trait Event {}
impl Event for winit::event::KeyboardInput {}

pub trait EventList {}
impl<H: Event> EventList for Events<H> {}

unsafe impl Sync for EventHandler {} 
unsafe impl Send for EventHandler {}
