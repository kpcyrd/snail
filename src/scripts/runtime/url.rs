use errors::{Result, Error};
use scripts::ctx::State;
use dns::DnsResolver;
use web::HttpClient;
use hlua;
use url::Url;
use std::sync::Arc;

pub fn url_join<C: HttpClient + 'static, R: DnsResolver + 'static>(lua: &mut hlua::Lua, state: Arc<State<C, R>>) {
    lua.set("url_join", hlua::function2(move |base: String, update: String| -> Result<String> {
        let base = Url::parse(&base)
            .map_err(|err| state.set_error(Error::from(err)))?;
        let url = base.join(&update)
            .map_err(|err| state.set_error(Error::from(err)))?;

        Ok(url.into_string())
    }))
}

#[cfg(test)]
mod tests {
    use scripts::loader::Loader;

    #[test]
    fn verify_relative_path() {
        let script = Loader::init_default(r#"
        descr = "verify_relative_path"

        function detect() end
        function decap()
            url = url_join("https://example.com/foo/abc", "bar")
            print(url)
            if url ~= "https://example.com/foo/bar" then
                return 'unexpected url'
            end
        end
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_relative_query() {
        let script = Loader::init_default(r#"
        descr = "verify_relative_query"

        function detect() end
        function decap()
            url = url_join("https://example.com/foo/", "?x=1&a=2")
            print(url)
            if url ~= "https://example.com/foo/?x=1&a=2" then
                return 'unexpected url'
            end
        end
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_relative_path_query() {
        let script = Loader::init_default(r#"
        descr = "verify_relative_path_query"

        function detect() end
        function decap()
            url = url_join("https://example.com/foo/bar", "abc?x=1&a=2")
            print(url)
            if url ~= "https://example.com/foo/abc?x=1&a=2" then
                return 'unexpected url'
            end
        end
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_absolute_path() {
        let script = Loader::init_default(r#"
        descr = "verify_absolute_path"

        function detect() end
        function decap()
            url = url_join("https://example.com/foo/abc", "/bar")
            print(url)
            if url ~= "https://example.com/bar" then
                return 'unexpected url'
            end
        end
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_absolute_query() {
        let script = Loader::init_default(r#"
        descr = "verify_absolute_query"

        function detect() end
        function decap()
            url = url_join("https://example.com/foo/", "/?x=1&a=2")
            print(url)
            if url ~= "https://example.com/?x=1&a=2" then
                return 'unexpected url'
            end
        end
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_absolute_path_query() {
        let script = Loader::init_default(r#"
        descr = "verify_absolute_path_query"

        function detect() end
        function decap()
            url = url_join("https://example.com/foo/abc", "/abc?x=1&a=2")
            print(url)
            if url ~= "https://example.com/abc?x=1&a=2" then
                return 'unexpected url'
            end
        end
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_replace() {
        let script = Loader::init_default(r#"
        descr = "verify_replace"

        function detect() end
        function decap()
            url = url_join("http://example.com/foo/?fizz=buzz", "https://asdf.com/abc?x=1&a=2")
            print(url)
            if url ~= "https://asdf.com/abc?x=1&a=2" then
                return 'unexpected url'
            end
        end
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_protocol_relative() {
        let script = Loader::init_default(r#"
        descr = "verify_protocol_relative"

        function detect() end
        function decap()
            url = url_join("https://example.com/foo/?fizz=buzz", "//asdf.com/abc?x=1&a=2")
            print(url)
            if url ~= "https://asdf.com/abc?x=1&a=2" then
                return 'unexpected url'
            end
        end
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }
}
