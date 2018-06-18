use scripts::ctx::State;
use dns::DnsResolver;
use web::HttpClient;
use hlua::{self, AnyLuaValue};
use std::sync::Arc;
use hlua::AnyHashableLuaValue;
use failure::ResultExt;
use std::collections::HashMap;
use web::structs::RequestOptions;
use web::structs::HttpRequest;
use ::{Result, Error};


pub fn http_mksession<C: HttpClient + 'static, R: DnsResolver + 'static>(lua: &mut hlua::Lua, state: Arc<State<C, R>>) {
    lua.set("http_mksession", hlua::function0(move || -> String {
        state.http_mksession()
    }))
}

pub fn http_request<C: HttpClient + 'static, R: DnsResolver + 'static>(lua: &mut hlua::Lua, state: Arc<State<C, R>>) {
    lua.set("http_request", hlua::function4(move |session: String, method: String, url: String, options: AnyLuaValue| -> Result<AnyLuaValue> {
        RequestOptions::try_from(options)
            .context("invalid request options")
            .map_err(|err| state.set_error(Error::from(err)))
            .map(|options| {
                state.http_request(&session, method, url, options).into()
            })
    }))
}

pub fn http_send<C: HttpClient + 'static, R: DnsResolver + 'static>(lua: &mut hlua::Lua, state: Arc<State<C, R>>) {
    lua.set("http_send", hlua::function1(move |request: AnyLuaValue| -> Result<HashMap<AnyHashableLuaValue, AnyLuaValue>> {
        let req = match HttpRequest::try_from(request)
                                .context("invalid http request object") {
            Ok(req) => req,
            Err(err) => return Err(state.set_error(Error::from(err))),
        };

        req.send(&state)
            .map_err(|err| state.set_error(err))
            .map(|resp| resp.into())
    }))
}

#[cfg(test)]
mod tests {
    use scripts::loader::Loader;

    #[test]
    #[ignore]
    fn verify_request() {
        let loader = Loader::default();

        let script = loader.load(r#"
        descr = "verify_request"

        function detect() end
        function decap()
            session = http_mksession()
            req = http_request(session, "GET", "https://httpbin.org/anything", {})
            x = http_send(req)
            print(x)
        end
        "#.to_string()).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    #[ignore]
    fn verify_post() {
        let loader = Loader::default();

        let script = loader.load(r#"
        descr = "verify_post"

        function detect() end
        function decap()
            session = http_mksession()

            headers = {}
            headers['Content-Type'] = "application/json"
            req = http_request(session, "POST", "https://httpbin.org/anything", {
                headers=headers,
                query={
                    foo="bar"
                },
                json={
                    hello="world"
                }
            })
            x = http_send(req)
            print(x)
        end
        "#.to_string()).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    #[ignore]
    fn verify_cookies() {
        let loader = Loader::default();

        let script = loader.load(r#"
        descr = "verify_request"

        function detect() end
        function decap()
            session = http_mksession()

            req = http_request(session, "GET", "https://httpbin.org/cookies/set", {
                query={
                    foo="bar",
                    fizz="buzz"
                }
            })
            x = http_send(req)

            req = http_request(session, "GET", "https://httpbin.org/cookies", {})
            x = http_send(req)
            print(x)
        end
        "#.to_string()).expect("failed to load script");
        script.decap().expect("decap failed");
    }
}
