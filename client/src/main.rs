use std::ops::Deref;

use log::debug;
use reqwasm::websocket::{futures::WebSocket, Message};
use sycamore::{prelude::*, rt::JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlInputElement};
#[derive(Default)]
struct AppState {
	chat_log: String,
	current_user: String,
}

fn main() {
	console_log::init_with_level(log::Level::Debug).expect("error initializing logger");
	let state = Signal::new(AppState::default());
	let socket = Signal::new(None);
	let on_click_join_user = cloned!((state,socket) => move |_ev: Event| {
		let current_user = state.get().current_user.to_owned();
		if !current_user.trim().is_empty() {
			let sock = reqwasm::websocket::futures::WebSocket::open("ws://localhost:3000/websocket").unwrap();
			socket.set(Some(sock));
		} else {
			debug!("Please add username");
		}
	});

	create_effect(cloned!((state, socket) => move || {
		if let Some(sock) = &*socket.get() {
			debug!("socket status{:?}", sock.state());
		}
	}));

	sycamore::render(|| {
		view! {
				h1 { "WebSocket Chat Example" }
				input(
					id="username",
					style="display:block; width:100px; box-sizing: border-box",
					type="text",
					placeholder="username",
					on:keyup=cloned!((state)=> move|ev:  Event| {
						let current_user = ev
							.target()
							.unwrap()
							.dyn_into::<HtmlInputElement>()
							.unwrap()
							.value();
						state.set(AppState {
							chat_log: String::from(state.get().chat_log.to_owned()),
							current_user,
						})
					})
				)
				button(
					id="join-chat",
					type="button",
					disabled=state.get().current_user.is_empty(),
					on:click=on_click_join_user,
				){"Join Chat"}
				textarea(id="chat", style="display:block; width:600px; height:400px; box-sizing: border-box", cols="30", rows="10")
				input(
					id="input",
					style="display:block; width:600px; box-sizing: border-box",
					type="text",
					placeholder="chat",
					disabled=socket.get().is_none()
				)
		}
	});
}
