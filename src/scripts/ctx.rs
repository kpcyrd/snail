use hlua;
use errors::{Result, Error};

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use dns::DnsResolver;
use scripts::runtime;
use web::HttpClient;
use web::structs::{HttpSession, HttpRequest, RequestOptions};


#[derive(Debug, Clone)]
pub struct State<C: HttpClient, R: DnsResolver> {
    error: Arc<Mutex<Option<Error>>>,
    http_sessions: Arc<Mutex<HashMap<String, HttpSession>>>,
    pub http: Arc<C>,
    pub resolver: Arc<R>,
}

impl<C: HttpClient, R: DnsResolver> State<C, R> {
    pub fn new(http: Arc<C>, resolver: Arc<R>) -> State<C, R> {
        State {
            error: Arc::new(Mutex::new(None)),
            http_sessions: Arc::new(Mutex::new(HashMap::new())),
            http,
            resolver,
        }
    }

    pub fn last_error(&self) -> Option<String> {
        let lock = self.error.lock().unwrap();
        lock.as_ref().map(|err| err.to_string())
    }

    pub fn set_error(&self, err: Error) -> Error {
        let mut mtx = self.error.lock().unwrap();
        let cp = format_err!("{:?}", err);
        *mtx = Some(err);
        cp.into()
    }

    pub fn http_mksession(&self) -> String {
        let mut mtx = self.http_sessions.lock().unwrap();
        let (id, session) = HttpSession::new();
        mtx.insert(id.clone(), session);
        id
    }

    pub fn http_request(&self, session_id: &str, method: String, url: String, options: RequestOptions) -> HttpRequest {
        let mtx = self.http_sessions.lock().unwrap();
        let session = mtx.get(session_id).expect("invalid session reference"); // TODO

        HttpRequest::new(&session, method, url, options)
    }

    pub fn register_in_jar(&self, session: &str, key: String, value: String) {
        let mut mtx = self.http_sessions.lock().unwrap();
        if let Some(session) = mtx.get_mut(session) {
            session.cookies.register_in_jar(key, value);
        }
    }
}


#[derive(Debug)]
pub struct Script<C: HttpClient, R: DnsResolver> {
    descr: String,
    code: String,
    http: Arc<C>,
    resolver: Arc<R>,
}

fn ctx<'a, C: HttpClient + 'static, R: DnsResolver + 'static>(http: Arc<C>, resolver: Arc<R>) -> (hlua::Lua<'a>, Arc<State<C, R>>) {
    let mut lua = hlua::Lua::new();
    lua.open_string();
    let state = Arc::new(State::new(http, resolver));

    runtime::dns(&mut lua, state.clone());
    runtime::html_select(&mut lua, state.clone());
    runtime::html_select_list(&mut lua, state.clone());
    runtime::http_mksession(&mut lua, state.clone());
    runtime::http_request(&mut lua, state.clone());
    runtime::http_send(&mut lua, state.clone());
    runtime::json_decode(&mut lua, state.clone());
    runtime::json_encode(&mut lua, state.clone());
    runtime::last_err(&mut lua, state.clone());
    runtime::print(&mut lua, state.clone());
    runtime::url_join(&mut lua, state.clone());

    (lua, state)
}

fn ensure_function_exists(lua: &mut hlua::Lua, name: &str) -> Result<()> {
    let func = match lua.get(name) {
        Some(func) => func,
        None => bail!("function is undefined: {:?}", name),
    };
    let _: hlua::LuaFunction<_> = func;
    Ok(())
}

impl<C: HttpClient + 'static, R: DnsResolver + 'static> Script<C, R> {
    pub fn load(code: String, http: Arc<C>, resolver: Arc<R>) -> Result<Script<C, R>> {
        let (mut lua, _) = ctx(http.clone(), resolver.clone());
        lua.execute::<()>(&code)?;

        let descr = {
            let descr: hlua::StringInLua<_> = match lua.get("descr") {
                Some(descr) => descr,
                None => bail!("descr undefined"),
            };
            (*descr).to_owned()
        };

        ensure_function_exists(&mut lua, "detect")?;
        ensure_function_exists(&mut lua, "decap")?;

        Ok(Script {
            descr,
            code,

            http,
            resolver,
        })
    }

    pub fn descr(&self) -> &str {
        &self.descr
    }

    pub fn detect_network(&self, network: &str) -> Result<bool> {
        let (mut lua, state) = ctx(self.http.clone(), self.resolver.clone());
        lua.execute::<()>(&self.code)?;

        let mut detect: hlua::LuaFunction<_> = match lua.get("detect") {
            Some(func) => func,
            None => bail!("function undefined: detect"),
        };

        let result: hlua::AnyLuaValue = match detect.call_with_args((network, )) {
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

    pub fn decap(&self) -> Result<()> {
        let (mut lua, state) = ctx(self.http.clone(), self.resolver.clone());
        lua.execute::<()>(&self.code)?;

        let mut decap: hlua::LuaFunction<_> = match lua.get("decap") {
            Some(func) => func,
            None => bail!("function undefined: decap"),
        };

        let result: hlua::AnyLuaValue = match decap.call() {
            Ok(res) => res,
            Err(err) => {
                bail!(format!("execution failed: {:?}", err));
            },
        };

        if let Some(err) = state.error.lock().unwrap().take() {
            return Err(err)
        }

        use hlua::AnyLuaValue::*;
        match result {
            LuaNil => Ok(()),
            LuaBoolean(true) => Ok(()),
            LuaBoolean(false) => Err(format_err!("script returned false")),
            LuaString(x) => Err(format_err!("error: {:?}", x)),
            x => Err(format_err!("lua returned wrong type: {:?}", x)),
        }
    }
}
