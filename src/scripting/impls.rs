use nalgebra::Vector3;
use mlua::{Lua, UserData, FromLua, ToLua, Value};
use crate::math::Transform;
use super::wrappers::LuaData;

impl<'lua> FromLua<'lua> for LuaData<Vector3<f32>> {
    fn from_lua(lua_value: Value<'lua>, _lua: &'lua Lua) -> mlua::Result<Self> {
        match lua_value {
            Value::Table(table) => {
                Ok(LuaData(Vector3::new(
                    table.get("x")?,
                    table.get("y")?,
                    table.get("z")?,
                )))
            }
            _ => {
                Err(mlua::Error::FromLuaConversionError { 
                    from: lua_value.type_name(), 
                    to: stringify!(Vector3<f32>), 
                    message: None,
                })
            }
        }
    }
}

impl<'lua> ToLua<'lua> for LuaData<Vector3<f32>> {
    fn to_lua(self, lua: &'lua Lua) -> mlua::Result<Value<'lua>> {
        Ok(Value::Table(
            lua.create_table_from([
                ("x", self.0.x),
                ("y", self.0.y),
                ("z", self.0.z),
            ])?
        ))
    }
}

impl UserData for Transform {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("translation", |_, this| {
            Ok(LuaData(this.translation))
        });

        fields.add_field_method_set("translation", |_, this, value: LuaData<Vector3<f32>>| {
            this.translation = *value;
            Ok(())
        });
    }
}