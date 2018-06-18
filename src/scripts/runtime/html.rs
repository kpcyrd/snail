use errors::Result;
use dns::DnsResolver;
use html;
use scripts::ctx::State;
use web::HttpClient;

use hlua::{self, AnyLuaValue};

use std::sync::Arc;


pub fn html_select<C: HttpClient + 'static, R: DnsResolver + 'static>(lua: &mut hlua::Lua, state: Arc<State<C, R>>) {
    lua.set("html_select", hlua::function2(move |html: String, selector: String| -> Result<AnyLuaValue> {
        html::html_select(&html, &selector)
            .map_err(|err| state.set_error(err))
            .map(|x| x.into())
    }))
}

pub fn html_select_list<C: HttpClient + 'static, R: DnsResolver + 'static>(lua: &mut hlua::Lua, state: Arc<State<C, R>>) {
    lua.set("html_select_list", hlua::function2(move |html: String, selector: String| -> Result<Vec<AnyLuaValue>> {
        html::html_select_list(&html, &selector)
            .map_err(|err| state.set_error(err))
            .map(|x| x.into_iter().map(|x| x.into()).collect())
    }))
}
