use errors::Result;
use scripts::ctx::State;
use json;
use dns::DnsResolver;
use web::HttpClient;

use hlua::{self, AnyLuaValue};

use std::sync::Arc;


pub fn json_decode<C: HttpClient + 'static, R: DnsResolver + 'static>(lua: &mut hlua::Lua, state: Arc<State<C, R>>) {
    lua.set("json_decode", hlua::function1(move |x: String| -> Result<AnyLuaValue> {
        json::decode(&x)
            .map_err(|err| state.set_error(err))
    }))
}

pub fn json_encode<C: HttpClient + 'static, R: DnsResolver + 'static>(lua: &mut hlua::Lua, state: Arc<State<C, R>>) {
    lua.set("json_encode", hlua::function1(move |x: AnyLuaValue| -> Result<String> {
        json::encode(x)
            .map_err(|err| state.set_error(err))
    }))
}
