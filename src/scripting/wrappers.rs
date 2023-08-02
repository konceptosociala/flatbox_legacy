use std::sync::Arc;
use std::ops::{Deref, DerefMut};

use parking_lot::RwLock;
use mlua::{Lua, UserData, ToLua};
use parking_lot::{Mutex, MutexGuard};
use super::script::Script;

pub struct LuaManager {
    inner: Arc<Mutex<Lua>>
}

impl LuaManager {
    pub fn new() -> Self {
        LuaManager { 
            inner: Arc::new(Mutex::new(Lua::new()))
        }
    }

    pub fn set_global<K: for<'lua> ToLua<'lua>, V: for<'lua> ToLua<'lua>>(
        &self, 
        key: K, 
        value: V,
    ) -> Result<(), mlua::Error> {
        let inner = self.inner();
        let globals = inner.globals();
        globals.set(key, value)
    }

    pub fn execute(&self, script: &Script) -> Result<(), mlua::Error> {
        self.inner().load(script.path.as_path()).exec()
    }

    fn inner(&self) -> MutexGuard<Lua> {
        self.inner.lock()
    }
}

#[derive(Clone)]
pub struct LuaData<T>(pub T);

impl<T> Deref for LuaData<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for LuaData<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct LuaPointer<T: UserData + Clone>(Arc<RwLock<*mut T>>);

impl<T: UserData + Clone> LuaPointer<T> {
    pub fn new(value: &mut T) -> LuaPointer<T> {
        LuaPointer(Arc::new(RwLock::new(value)))
    }

    pub fn get(&self) -> T {
        unsafe { (**self.0.read()).clone() }
    }

    pub fn set(&mut self, value: T) {
        unsafe { **self.0.write() = value }
    }
}

impl<T: UserData + Send + Sync + Clone + 'static> UserData for LuaPointer<T> {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("get", |_, this| {
            Ok(this.get())
        });
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("set", |_, this, value: T| {
            this.set(value.clone());
            Ok(())
        });
    }
}

unsafe impl<T: UserData + Send + Sync + Clone> Send for LuaPointer<T> {}
unsafe impl<T: UserData + Send + Sync + Clone> Sync for LuaPointer<T> {}