use errors::Result;
use dns::DnsResolver;
use html;
use scripts::ctx::State;
use structs::LuaMap;
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

pub fn html_meta_refresh<C: HttpClient + 'static, R: DnsResolver + 'static>(lua: &mut hlua::Lua, state: Arc<State<C, R>>) {
    lua.set("html_meta_refresh", hlua::function1(move |html: String| -> Result<String> {
        let meta = html::html_select(&html, "meta[http-equiv=\"refresh\"]")
            .map_err(|err| state.set_error(err))?;

        let content = match meta.attrs.get("content") {
            Some(content) => content,
            None => return Err(state.set_error(format_err!("content attribute not found"))),
        };

        let url = match content.find(";") {
            Some(idx) => content[idx+1..].trim(),
            None => return Err(state.set_error(format_err!("content has no attributes"))),
        };

        if url.starts_with("url=") {
            Ok(url[4..].to_string())
        } else {
            Err(state.set_error(format_err!("url= attribute not found")))
        }
    }))
}

pub fn html_form<C: HttpClient + 'static, R: DnsResolver + 'static>(lua: &mut hlua::Lua, state: Arc<State<C, R>>) {
    lua.set("html_form", hlua::function1(move |html: String| -> Result<AnyLuaValue> {
        html::html_form(&html)
            .map_err(|err| state.set_error(err))
            .map(|x| LuaMap::from(x).into())
    }))
}

#[cfg(test)]
mod tests {
    use scripts::loader::Loader;

    #[test]
    fn verify_html_select() {
        let script = Loader::init_default(r#"
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
            if x['html'] ~= '<div id="yey">content</div>' then
                return 'wrong html'
            end
        end
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_html_select_list() {
        let script = Loader::init_default(r#"
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
            if x[1]['html'] ~= '<div id="yey">content</div>' then
                return 'wrong html'
            end
        end
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_html_meta_refresh() {
        let script = Loader::init_default(r#"
        descr = "html_meta_refresh"

        function detect() end
        function decap()
            x = html_meta_refresh('<meta http-equiv="refresh" content="2;url=https://example.com/foo" />')
            if last_err() then return end
            print(x)
            if x ~= "https://example.com/foo" then
                return "https://example.com/foo expected"
            end

            x = html_meta_refresh('<meta http-equiv="refresh" content="120;   url=?asdf=1&x=y" />')
            if last_err() then return end
            print(x)
            if x ~= "?asdf=1&x=y" then
                return "?asdf=1&x=y expected"
            end

            x = html_meta_refresh('<meta http-equiv="refresh" content="0; \n url=/foo/bar">')
            if last_err() then return end
            print(x)
            if x ~= "/foo/bar" then
                return "/foo/bar expected"
            end
        end
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_html_meta_refresh_missing_url() {
        let script = Loader::init_default(r#"
        descr = "html_meta_refresh"

        function detect() end
        function decap()
            x = html_meta_refresh('<meta http-equiv="refresh" content="2" />')
            if last_err() then return end
        end
        "#).expect("failed to load script");
        let x = script.decap();
        assert!(x.is_err());
    }

    #[test]
    fn verify_html_form() {
        let script = Loader::init_default(r#"
        descr = "html_form"

        function detect() end
        function decap()
            x = html_form('<form method="POST">\n' ..
                          '    <input type="hidden" name="foo_hidden" value="bar">\n' ..
                          '    <input type="submit" name="foo_submit" value="asdf">\n' ..
                          '</form>')
            if last_err() then return end

            if x['foo_hidden'] ~= "bar" then
                return 'wrong foo_hidden'
            end
            if x['foo_submit'] ~= "asdf" then
                return 'wrong foo_submit'
            end
        end
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }
}
