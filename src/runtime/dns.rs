use errors::{Result, Error};
use scripts::ctx::State;
use dns::DnsResolver;
use web::HttpClient;
use hlua;
use std::sync::Arc;

pub fn dns<C: HttpClient + 'static, R: DnsResolver + 'static>(lua: &mut hlua::Lua, state: Arc<State<C, R>>) {
    lua.set("dns", hlua::function1(move |name: String| -> Result<Vec<String>> {
        let x = state.resolver.resolve(&name)
            .map_err(|err| state.set_error(Error::from(err)))?;
        Ok(x.into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>())
    }))
}

#[cfg(test)]
mod tests {
    use scripts::loader::Loader;

    #[test]
    #[ignore]
    fn verify_resolve() {
        let script = Loader::init_default(r#"
        descr = "verify_resolve"

        function detect() end
        function decap()
            x = dns("google.com")
            print(x)
        end
        "#).expect("failed to load script");
        script.decap().expect("decap failed");
    }
}
