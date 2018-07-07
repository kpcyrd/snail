use hlua::{self, AnyLuaValue};
use scripts::ctx::State;
use dns::DnsResolver;
use web::HttpClient;

use std::sync::Arc;

pub fn last_err<C: HttpClient + 'static, R: DnsResolver + 'static>(lua: &mut hlua::Lua, state: Arc<State<C, R>>) {
    lua.set("last_err", hlua::function0(move || -> AnyLuaValue {
        match state.last_error() {
            Some(err) => AnyLuaValue::LuaString(err),
            None => AnyLuaValue::LuaNil,
        }
    }))
}
