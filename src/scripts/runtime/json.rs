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

#[cfg(test)]
mod tests {
    use scripts::loader::Loader;

    #[test]
    fn verify_json_encode() {
        let loader = Loader::default();

        let script = loader.load(r#"
        descr = "json"

        function detect() end
        function decap()
            json_encode({
                hello="world",
                almost_one=0.9999,
                list={1,3,3,7},
                data={
                    user=user,
                    password=password,
                    empty=nil
                }
            })
        end
        "#.to_string()).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_json_encode_decode() {
        let loader = Loader::default();

        let script = loader.load(r#"
        descr = "json"

        function detect() end
        function decap()
            x = json_encode({
                hello="world",
                almost_one=0.9999,
                list={1,3,3,7},
                data={
                    user=user,
                    password=password,
                    empty=nil
                }
            })
            json_decode(x)
        end
        "#.to_string()).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_json_decode_valid() {
        let loader = Loader::default();

        let script = loader.load(r#"
        descr = "json"

        function detect() end
        function decap()
            json_decode("{\"almost_one\":0.9999,\"data\":{\"password\":\"fizz\",\"user\":\"bar\"},\"hello\":\"world\",\"list\":[1,3,3,7]}")
        end
        "#.to_string()).expect("failed to load script");
        script.decap().expect("decap failed");
    }

    #[test]
    fn verify_json_decode_invalid() {
        let loader = Loader::default();

        let script = loader.load(r#"
        descr = "json"

        function detect() end
        function decap()
            json_decode("{\"almost_one\":0.9999,\"data\":{\"password\":\"fizz\",\"user\":\"bar\"}}}}}}}}}")
        end
        "#.to_string()).expect("failed to load script");
        let r = script.decap();
        assert!(r.is_err());
    }
}
