use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use errors::Result;
use hlua::AnyLuaValue;
use scripts::ctx::State;
use serde_json;
use dns::DnsResolver;
use web::HttpClient;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use structs::LuaMap;
use json::LuaJsonValue;
use http::Uri;
use http::uri::Parts;
use hyper::{Request, Body};
use serde_urlencoded;
use base64;


#[derive(Debug)]
pub struct HttpSession {
    id: String,
    pub cookies: CookieJar,
}

impl HttpSession {
    pub fn new() -> (String, HttpSession) {
        let id: String = thread_rng().sample_iter(&Alphanumeric).take(16).collect();
        (id.clone(), HttpSession {
            id,
            cookies: CookieJar::default(),
        })
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct RequestOptions {
    query: Option<HashMap<String, String>>,
    headers: Option<HashMap<String, String>>,
    basic_auth: Option<(String, String)>,
    user_agent: Option<String>,
    json: Option<serde_json::Value>,
    form: Option<serde_json::Value>,
    body: Option<String>,
}

impl RequestOptions {
    pub fn try_from(x: AnyLuaValue) -> Result<RequestOptions> {
        let x = LuaJsonValue::from(x);
        let x = serde_json::from_value(x.into())?;
        Ok(x)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HttpRequest {
    // reference to the HttpSession
    session: String,
    cookies: CookieJar,
    method: String,
    url: String,
    query: Option<HashMap<String, String>>,
    headers: Option<HashMap<String, String>>,
    basic_auth: Option<(String, String)>,
    user_agent: Option<String>,
    body: Option<ReqBody>,
}

impl HttpRequest {
    pub fn new(session: &HttpSession, method: String, url: String, options: RequestOptions) -> HttpRequest {
        let cookies = session.cookies.clone();

        let user_agent = options.user_agent.or_else(|| Some("snail agent".to_string())); // TODO

        let mut request = HttpRequest {
            session: session.id.clone(),
            cookies,
            method,
            url,
            query: options.query,
            headers: options.headers,
            basic_auth: options.basic_auth,
            user_agent,
            body: None,
        };

        if let Some(json) = options.json {
            request.body = Some(ReqBody::Json(json));
        }

        if let Some(form) = options.form {
            request.body = Some(ReqBody::Form(form));
        }

        if let Some(text) = options.body {
            request.body = Some(ReqBody::Raw(text));
        }

        request
    }

    pub fn send<C: HttpClient, R: DnsResolver>(&self, state: &State<C, R>) -> Result<LuaMap> {
        let mut url = self.url.parse::<Uri>()?;

        // set query string
        if let Some(query) = &self.query {
            url = {
                let mut parts = Parts::from(url);

                let query = serde_urlencoded::to_string(query)?;

                parts.path_and_query = Some(match parts.path_and_query {
                    Some(pq) => {
                        format!("{}?{}", pq.path(), query)
                    },
                    None => format!("/?{}", query),
                }.parse()?);

                Uri::from_parts(parts)?
            };
        }

        // start setting up request
        let mut req = Request::builder();
        req.method(self.method.as_str());
        req.uri(url.clone());

        let mut observed_headers = HashSet::new();

        // set cookies
        {
            use hyper::header::COOKIE;
            let mut cookies = String::new();

            for (key, value) in self.cookies.iter() {
                if !cookies.is_empty() {
                    cookies += "; ";
                }
                cookies.push_str(&format!("{}={}", key, value));
            }

            if !cookies.is_empty() {
                req.header(COOKIE, cookies.as_str());
                observed_headers.insert(COOKIE.as_str().to_lowercase());
            }
        }

        // add headers
        if let Some(ref agent) = self.user_agent {
            use hyper::header::USER_AGENT;
            req.header(USER_AGENT, agent.as_str());
            observed_headers.insert(USER_AGENT.as_str().to_lowercase());
        }

        if let Some(ref auth) = self.basic_auth {
            use hyper::header::AUTHORIZATION;
            let &(ref user, ref password) = auth;

            let auth = base64::encode(&format!("{}:{}", user, password));
            let auth = format!("Basic {}", auth);
            req.header(AUTHORIZATION, auth.as_str());
            observed_headers.insert(AUTHORIZATION.as_str().to_lowercase());
        }

        if let Some(ref headers) = self.headers {
            for (k, v) in headers {
                req.header(k.as_str(), v.as_str());
                observed_headers.insert(k.to_lowercase());
            }
        }

        // finalize request
        let body = match self.body {
            Some(ReqBody::Raw(ref x))  => { Body::from(x.clone()) },
            Some(ReqBody::Form(ref x)) => {
                // if Content-Type is not set, set header
                if !observed_headers.contains("content-type") {
                    req.header("Content-Type", "application/x-www-form-urlencoded");
                }
                Body::from(serde_urlencoded::to_string(x)?)
            },
            Some(ReqBody::Json(ref x)) => {
                // if Content-Type is not set, set header
                if !observed_headers.contains("content-type") {
                    req.header("Content-Type", "application/json");
                }
                Body::from(serde_json::to_string(x)?)
            },
            None => Body::empty(),
        };
        let req = req.body(body)?;

        // send request
        let res = state.http.request(&url, req)?;

        // map result to LuaMap
        let mut resp = LuaMap::new();
        resp.insert_num("status", f64::from(res.status));

        for cookie in &res.cookies {
            HttpRequest::register_cookies_on_state(&self.session, state, cookie);
        }

        let mut headers = LuaMap::new();
        for (key, value) in res.headers {
            headers.insert_str(key.to_lowercase(), value);
        }
        resp.insert("headers", headers);

        resp.insert_str("text", res.body);

        Ok(resp)
    }

    fn register_cookies_on_state<C: HttpClient, R: DnsResolver>(session: &str, state: &State<C, R>, cookie: &str) {
        let mut key = String::new();
        let mut value = String::new();
        let mut in_key = true;

        for c in cookie.as_bytes() {
            match *c as char {
                '=' if in_key => in_key = false,
                ';' => break,
                c if in_key => key.push(c),
                c => value.push(c),
            }
        }

        state.register_in_jar(session, key, value);
    }
}

impl HttpRequest {
    pub fn try_from(x: AnyLuaValue) -> Result<HttpRequest> {
        let x = LuaJsonValue::from(x);
        let x = serde_json::from_value(x.into())?;
        Ok(x)
    }
}

impl Into<AnyLuaValue> for HttpRequest {
    fn into(self) -> AnyLuaValue {
        let v = serde_json::to_value(&self).unwrap();
        LuaJsonValue::from(v).into()
    }
}

// see https://github.com/seanmonstar/reqwest/issues/14 for proper cookie jars
// maybe change this to reqwest::header::Cookie
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CookieJar(HashMap<String, String>);

impl CookieJar {
    pub fn register_in_jar(&mut self, key: String, value: String) {
        self.0.insert(key, value);
    }
}

impl Deref for CookieJar {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ReqBody {
    Raw(String), // TODO: maybe Vec<u8>
    Form(serde_json::Value),
    Json(serde_json::Value),
}
