//! The browser host for the state machine (wasm32 only): WebSockets to the
//! relays, one timer, WebCrypto for the seat's Ed25519 key, `fetch` for the
//! NIP-11 screen. Everything that decides anything is in `session`; this
//! file carries out its `Output`s and feeds it what happens.
//!
//! The seat key is generated non-extractable (sessions spec §4.1), so its
//! private half exists only inside WebCrypto: this module asks WebCrypto to
//! sign and passes the signature back.

use crate::link::Params;
use crate::session::{Client, Output};
use js_sys::{Array, Date, Function, Object, Reflect, Uint8Array};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{CryptoKey, CryptoKeyPair, MessageEvent, Request, RequestInit, Response, WebSocket};

struct Socket {
    ws: WebSocket,
    _on_open: Closure<dyn FnMut()>,
    _on_message: Closure<dyn FnMut(MessageEvent)>,
    _on_close: Closure<dyn FnMut()>,
}

struct Inner {
    client: Client,
    key: CryptoKey,
    sockets: Vec<Option<Socket>>,
    /// Bumped when a relay's socket is replaced, so a late event from an old
    /// socket is ignored.
    generation: Vec<u32>,
    timer: Option<i32>,
    timer_cb: Option<Closure<dyn FnMut()>>,
    on_change: Option<Function>,
}

type Shared = Rc<RefCell<Inner>>;

/// A JS error as one line: its message if it has one.
fn js_err(e: &JsValue) -> String {
    Reflect::get(e, &"message".into())
        .ok()
        .and_then(|m| m.as_string())
        .or_else(|| e.as_string())
        .unwrap_or_else(|| "unknown error".into())
}

fn now() -> f64 {
    Date::now()
}

fn window() -> web_sys::Window {
    web_sys::window().expect("a window")
}

fn subtle() -> web_sys::SubtleCrypto {
    window().crypto().expect("WebCrypto").subtle()
}

fn ed25519() -> Object {
    let o = Object::new();
    Reflect::set(&o, &"name".into(), &"Ed25519".into()).expect("set");
    o
}

/// Carries out everything the state machine asks for, then sets the timer
/// for its next wake and tells the page.
fn pump(shared: &Shared) {
    loop {
        let outputs = shared.borrow_mut().client.poll(now());
        if outputs.is_empty() {
            break;
        }
        for o in outputs {
            match o {
                Output::Connect(i, url) => connect(shared, i, &url),
                Output::Send(i, text) => {
                    let mut inner = shared.borrow_mut();
                    let failed = match &inner.sockets[i] {
                        Some(s) => s.ws.send_with_str(&text).is_err(),
                        None => true,
                    };
                    if failed {
                        inner.client.host_note(Some(i), "send-failed", "the socket was not open", now());
                    }
                }
                Output::Sign(req, bytes) => sign(shared, req, bytes),
            }
        }
    }
    schedule(shared);
    let cb = shared.borrow().on_change.clone();
    if let Some(cb) = cb {
        let _ = cb.call0(&JsValue::NULL);
    }
}

fn schedule(shared: &Shared) {
    let mut inner = shared.borrow_mut();
    if let Some(t) = inner.timer.take() {
        window().clear_timeout_with_handle(t);
    }
    let t = now();
    if let Some(at) = inner.client.next_wake(t) {
        let ms = (at - t).clamp(0.0, 600_000.0) as i32;
        let cb = inner.timer_cb.as_ref().expect("timer callback").as_ref().unchecked_ref::<Function>().clone();
        inner.timer = window().set_timeout_with_callback_and_timeout_and_arguments_0(&cb, ms).ok();
    }
}

fn connect(shared: &Shared, i: usize, url: &str) {
    let weak: Weak<RefCell<Inner>> = Rc::downgrade(shared);
    let ws = match WebSocket::new(url) {
        Ok(ws) => ws,
        Err(e) => {
            let mut inner = shared.borrow_mut();
            inner.client.host_note(Some(i), "socket-error", &js_err(&e), now());
            inner.client.relay_closed(i, now());
            return;
        }
    };
    let gen = {
        let mut inner = shared.borrow_mut();
        inner.generation[i] += 1;
        inner.generation[i]
    };
    let live = move |w: &Weak<RefCell<Inner>>| -> Option<Shared> {
        let s = w.upgrade()?;
        let ok = s.borrow().generation[i] == gen;
        ok.then_some(s)
    };
    let (w1, w2, w3) = (weak.clone(), weak.clone(), weak);
    let on_open = Closure::<dyn FnMut()>::new(move || {
        if let Some(s) = live(&w1) {
            s.borrow_mut().client.relay_opened(i, now());
            pump(&s);
        }
    });
    let on_message = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
        if let (Some(s), Some(text)) = (live(&w2), e.data().as_string()) {
            s.borrow_mut().client.relay_text(i, &text, now());
            pump(&s);
        }
    });
    let on_close = Closure::<dyn FnMut()>::new(move || {
        if let Some(s) = live(&w3) {
            {
                let mut inner = s.borrow_mut();
                inner.sockets[i] = None;
                inner.client.relay_closed(i, now());
            }
            pump(&s);
        }
    });
    ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
    shared.borrow_mut().sockets[i] =
        Some(Socket { ws, _on_open: on_open, _on_message: on_message, _on_close: on_close });
}

fn sign(shared: &Shared, req: u64, bytes: Vec<u8>) {
    let key = shared.borrow().key.clone();
    let weak = Rc::downgrade(shared);
    spawn_local(async move {
        let data = Uint8Array::from(bytes.as_slice());
        let result = match subtle().sign_with_object_and_buffer_source(&ed25519(), &key, &data) {
            Ok(p) => JsFuture::from(p).await,
            Err(e) => Err(e),
        };
        let Some(s) = weak.upgrade() else { return };
        match result {
            Ok(buf) => {
                let v = Uint8Array::new(&buf).to_vec();
                match <[u8; 64]>::try_from(v.as_slice()) {
                    Ok(sig) => s.borrow_mut().client.signed(req, sig, now()),
                    Err(_) => s.borrow_mut().client.host_note(None, "sign-failed", "signature was not 64 bytes", now()),
                }
            }
            Err(e) => s.borrow_mut().client.host_note(None, "sign-failed", &js_err(&e), now()),
        }
        pump(&s);
    });
}

/// The NIP-11 screen (§4.3.1 rule 3): a relay that asks for proof-of-work,
/// authentication or payment, or cannot hold a bundle of frames, is read and
/// never written to. A relay whose document cannot be fetched is kept.
fn screen(shared: &Shared, i: usize, url: String) {
    let weak = Rc::downgrade(shared);
    spawn_local(async move {
        let https = url.replacen("wss://", "https://", 1).replacen("ws://", "http://", 1);
        let verdict: Result<(bool, String), String> = async {
            let init = RequestInit::new();
            let req = Request::new_with_str_and_init(&https, &init).map_err(|e| js_err(&e))?;
            req.headers().set("Accept", "application/nostr+json").map_err(|e| js_err(&e))?;
            let resp: Response = JsFuture::from(window().fetch_with_request(&req))
                .await
                .map_err(|e| js_err(&e))?
                .dyn_into()
                .map_err(|_| "not a response")?;
            let json = JsFuture::from(resp.json().map_err(|e| js_err(&e))?).await.map_err(|e| js_err(&e))?;
            let lim = Reflect::get(&json, &"limitation".into()).unwrap_or(JsValue::UNDEFINED);
            let num = |k: &str| Reflect::get(&lim, &k.into()).ok().and_then(|v| v.as_f64());
            let flag = |k: &str| Reflect::get(&lim, &k.into()).ok().and_then(|v| v.as_bool()).unwrap_or(false);
            let pow = num("min_pow_difficulty").unwrap_or(0.0);
            let content = num("max_content_length");
            let mut reasons = vec![];
            if pow > 0.0 {
                reasons.push(format!("proof-of-work {pow}"));
            }
            if flag("auth_required") {
                reasons.push("authentication".into());
            }
            if flag("payment_required") {
                reasons.push("payment".into());
            }
            if content.is_some_and(|c| c < 2048.0) {
                reasons.push(format!("max_content_length {}", content.unwrap_or(0.0)));
            }
            let software = Reflect::get(&json, &"software".into()).ok().and_then(|v| v.as_string()).unwrap_or_default();
            Ok(if reasons.is_empty() {
                (true, format!("NIP-11 ok {software}"))
            } else {
                (false, format!("requires {}", reasons.join(", ")))
            })
        }
        .await;
        let Some(s) = weak.upgrade() else { return };
        match verdict {
            Ok((usable, note)) => s.borrow_mut().client.screen(i, usable, &note, now()),
            Err(e) => s.borrow_mut().client.host_note(
                Some(i),
                "screen-unavailable",
                &format!("NIP-11 fetch failed, relay kept: {e}"),
                now(),
            ),
        }
        pump(&s);
    });
}

/// The page's handle on the session.
#[wasm_bindgen]
pub struct Net {
    shared: Shared,
}

/// Generates the seat key in WebCrypto and starts a session over `relays`.
#[wasm_bindgen]
pub async fn start_net(relays: Array) -> Result<Net, JsValue> {
    let urls: Vec<String> = relays.iter().filter_map(|v| v.as_string()).collect();
    let pair: CryptoKeyPair = JsFuture::from(subtle().generate_key_with_object(
        &ed25519(),
        false,
        &Array::of2(&"sign".into(), &"verify".into()),
    )?)
    .await?
    .unchecked_into();
    let public: CryptoKey = Reflect::get(&pair, &"publicKey".into())?.unchecked_into();
    let private: CryptoKey = Reflect::get(&pair, &"privateKey".into())?.unchecked_into();
    let raw = Uint8Array::new(&JsFuture::from(subtle().export_key("raw", &public)?).await?).to_vec();
    let me: [u8; 32] = raw.try_into().map_err(|_| JsValue::from_str("the public key is not 32 bytes"))?;
    let mut seed = [0u8; 32];
    getrandom::getrandom(&mut seed).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let n = urls.len();
    let client = Client::new(urls.clone(), me, seed, now());
    let shared: Shared = Rc::new(RefCell::new(Inner {
        client,
        key: private,
        sockets: (0..n).map(|_| None).collect(),
        generation: vec![0; n],
        timer: None,
        timer_cb: None,
        on_change: None,
    }));
    let weak = Rc::downgrade(&shared);
    let tick = Closure::<dyn FnMut()>::new(move || {
        if let Some(s) = weak.upgrade() {
            s.borrow_mut().timer = None;
            pump(&s);
        }
    });
    shared.borrow_mut().timer_cb = Some(tick);
    for (i, url) in urls.into_iter().enumerate() {
        screen(&shared, i, url);
    }
    Ok(Net { shared })
}

#[wasm_bindgen]
impl Net {
    /// Called after anything changes; the page redraws from `view()`.
    pub fn on_change(&self, f: Function) {
        self.shared.borrow_mut().on_change = Some(f);
    }

    /// The whole state the page shows, as JSON.
    pub fn view(&self) -> String {
        self.shared.borrow().client.view(now()).to_string()
    }

    pub fn browse(&self) {
        self.shared.borrow_mut().client.browse(now());
        pump(&self.shared);
    }

    /// Makes a room; returns the link payload (everything after `#j=`).
    pub fn create(&self, seats: u8, round_s: u16, rounds: u8, patience_s: u16, public: bool, name: &str) -> String {
        let p = Params { seats, round_s, rounds, patience_s, public };
        let link = self.shared.borrow_mut().client.create(p, name, now());
        pump(&self.shared);
        link
    }

    pub fn open_link(&self, link: &str) -> Result<(), JsValue> {
        let r = self.shared.borrow_mut().client.open_link(link, now());
        pump(&self.shared);
        r.map_err(|e| JsValue::from_str(&e))
    }

    pub fn join(&self, name: &str) -> Result<(), JsValue> {
        let r = self.shared.borrow_mut().client.join(name, now());
        pump(&self.shared);
        r.map_err(|e| JsValue::from_str(&e))
    }

    pub fn kick(&self, key_hex: &str) -> Result<(), JsValue> {
        let r = self.shared.borrow_mut().client.kick(key_hex, now());
        pump(&self.shared);
        r.map_err(|e| JsValue::from_str(&e))
    }

    pub fn start(&self) -> Result<(), JsValue> {
        let r = self.shared.borrow_mut().client.start(now());
        pump(&self.shared);
        r.map_err(|e| JsValue::from_str(&e))
    }

    pub fn accept(&self) -> Result<(), JsValue> {
        let r = self.shared.borrow_mut().client.accept(now());
        pump(&self.shared);
        r.map_err(|e| JsValue::from_str(&e))
    }

    pub fn set_auto(&self, auto: bool) {
        self.shared.borrow_mut().client.set_auto(auto);
        pump(&self.shared);
    }

    pub fn choose(&self, card: u16) {
        self.shared.borrow_mut().client.choose(card);
        pump(&self.shared);
    }

    /// Every relay reply and protocol event, TSV (§4.3.2 rule 8).
    pub fn diagnostics(&self) -> String {
        self.shared.borrow().client.diagnostics_tsv()
    }

    /// The match record (§5.4), JSON.
    pub fn transcript(&self) -> String {
        self.shared.borrow().client.transcript_json()
    }
}
