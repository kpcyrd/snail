use hlua;
use ::{Result, Error};
use failure::ResultExt;

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};


#[derive(Debug)]
pub struct State {
    error: Arc<Mutex<Option<Error>>>,
}

impl State {
    pub fn new() -> State {
        State {
            error: Arc::new(Mutex::new(None)),
        }
    }
}


#[derive(Debug)]
pub struct Script {
    descr: String,
    code: String,
}

impl Script {
    pub fn load_from_path(path: PathBuf) -> Result<Script> {
        info!("loading script from: {:?}", path);
        let code = fs::read_to_string(&path)?;
        let script = Script::load(code)
                        .context(format!("failed to load {:?}", path))?;
        Ok(script)
    }

    pub fn load(code: String) -> Result<Script> {
        let (mut lua, _) = Script::ctx();
        lua.execute::<()>(&code)?;

        let descr = {
            let descr: hlua::StringInLua<_> = match lua.get("descr") {
                Some(descr) => descr,
                None => bail!("descr undefined"),
            };
            (*descr).to_owned()
        };

        Script::ensure_function_exists(&mut lua, "detect")?;
        Script::ensure_function_exists(&mut lua, "decap")?;

        Ok(Script {
            descr,
            code,
        })
    }

    fn ensure_function_exists(lua: &mut hlua::Lua, name: &str) -> Result<()> {
        let func = match lua.get(name) {
            Some(func) => func,
            None => bail!("function is undefined: {:?}", name),
        };
        let _: hlua::LuaFunction<_> = func;
        Ok(())
    }

    fn ctx<'a>() -> (hlua::Lua<'a>, State) {
        let mut lua = hlua::Lua::new();
        lua.open_string();
        let state = State::new();

        (lua, state)
    }

    pub fn detect_network(&self, network: &str) -> Result<bool> {
        let (mut lua, state) = Script::ctx();
        lua.execute::<()>(&self.code)?;

        let mut verify: hlua::LuaFunction<_> = match lua.get("detect") {
            Some(func) => func,
            None => bail!("function undefined: detect"),
        };

        let result: hlua::AnyLuaValue = match verify.call_with_args((network, )) {
            Ok(res) => res,
            Err(err) => {
                bail!(format!("execution failed: {:?}", err));
            },
        };

        if let Some(err) = state.error.lock().unwrap().take() {
            return Err(err);
        }

        use hlua::AnyLuaValue::*;
        match result {
            LuaBoolean(x) => Ok(x),
            LuaString(x) => bail!(format!("error: {:?}", x)),
            x => bail!(format!("lua returned wrong type: {:?}", x)),
        }
    }
}
