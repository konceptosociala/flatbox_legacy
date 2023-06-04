use std::collections::HashMap;
use std::any::TypeId;
use std::sync::Arc;

use parking_lot::{Mutex, MutexGuard, MappedMutexGuard};
use as_any::AsAny;

/// App exit event. When it's sent, application makes close request
#[derive(Clone)]
pub struct AppExit;

/// Generic event trait. Every clonable `Send` + `Sync` type can be `Event`
pub trait Event: Clone + Send + Sync + 'static {}
impl<E: Clone + Send + Sync + 'static> Event for E {}

/// Routine, which reads and writes events of a concrete type
pub struct EventHandler<E: Event> {
    events: Option<E>,
}

impl<E: Event> EventHandler<E> {
    /// Instantiate new empty [`EventHandler`]
    pub fn new() -> Self {
        EventHandler::<E>::default()
    }
    
    /// Send event to the handler
    pub fn send(&mut self, event: E){        
        self.events = Some(event);
    }
    
    /// Listen for events
    pub fn read(&self) -> Option<E> {
        self.events.clone()
    }
    
    /// Clear events. It is called by the engine at every schedule run
    pub fn clear(&mut self){
        self.events = None;
    }
}

impl<E: Event> Default for EventHandler<E> {
    fn default() -> Self {
        EventHandler { events: None }
    }
}

pub trait GenericEventHandler: AsAny + Send + Sync + 'static {}
impl<E: Event> GenericEventHandler for EventHandler<E> {}

#[derive(Default)]
pub struct Events {
    storage: HashMap<TypeId, Arc<Mutex<dyn GenericEventHandler>>>,
}

impl Events {
    pub fn new() -> Self {
        Events::default()
    }

    pub fn get_handler<E: Event>(&self) -> Option<MappedMutexGuard<EventHandler<E>>> { 
        if let Some(handler) = self.storage.get(&TypeId::of::<EventHandler<E>>()){
            let data = handler.lock();
            return MutexGuard::try_map(data, |data| {
                data.as_any_mut().downcast_mut::<EventHandler<E>>()
            }).ok() 
        }

        None
    }

    pub fn push_handler<H: GenericEventHandler>(
        &mut self,
        handler: H,
    ){
        if self.storage.contains_key(&TypeId::of::<H>()) {
            log::error!("Event handler '{}' is already pushed!", std::any::type_name::<H>());
        } else {
            self.storage.insert(TypeId::of::<H>(), Arc::new(Mutex::new(handler)));
        }
    }
}