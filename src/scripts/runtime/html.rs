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

#[cfg(test)]
mod tests {
    use scripts::loader::Loader;

    #[test]
    fn verify_html_select() {
        let loader = Loader::default();

        let script = loader.load(r#"
        descr = "html"

        function detect() end
        function decap()
            x = html_select('<html><div id="yey">content</div></html>', '#yey')
            print(x)
            if x['text'] ~= 'content' then
                return 'wrong text'
            end
            if x['attrs']['id'] ~= 'yey' then
                return 'wrong id'
            end
        end
        "#.to_string()).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_html_select_list() {
        let loader = Loader::default();

        let script = loader.load(r#"
        descr = "html"

        function detect() end
        function decap()
            x = html_select_list('<html><div id="yey">content</div></html>', '#yey')
            print(x)
            if x[1] == nil then
                return 'wrong number of results'
            end
            if x[1]['text'] ~= 'content' then
                return 'wrong text'
            end
            if x[1]['attrs']['id'] ~= 'yey' then
                return 'wrong id'
            end
        end
        "#.to_string()).expect("failed to load script");
        script.decap().expect("decap failed");
    }
}
