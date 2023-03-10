pub struct EventHandler<E: Clone + Send + Sync> {
	events: Option<E>,
}

impl<E: Clone + Send + Sync> EventHandler<E> {
	pub fn new() -> Self {
		Self { 
			events: None,
		}
	}
	
	pub fn send(&mut self, event: E){		
		self.events = Some(event);
	}
    
    pub fn read(&self) -> Option<E> {
        self.events.clone()
    }
    
    pub fn clear(&mut self){
        self.events = None;
    }
}

#[derive(Clone)]
pub struct AppExit;
