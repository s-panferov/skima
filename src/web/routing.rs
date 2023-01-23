use std::rc::Rc;

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

use super::dispatch::DispatcherExt;
use crate::action::Action;
use crate::web::{Markup, WebSys};

pub struct HistoryEvent(Closure<dyn Fn(web_sys::Event)>);

struct History {}

#[derive(Debug)]
pub struct Navigate(pub String);

impl Action for Navigate {}

impl Markup for History {
	fn has_own_node() -> bool {
		false
	}

	fn render(&self, tree: &crate::tree::Tree<WebSys>) {
		let callback: Closure<dyn Fn(web_sys::Event)> = Closure::new({
			let tree = tree.clone();
			move |event: web_sys::Event| {
				let Some(href) = event
					.target()
					.unwrap()
					.unchecked_into::<web_sys::Element>()
					.closest("[href]")
					.unwrap()
					.and_then(|el| el.get_attribute("href")) else {
						return
					};

				let history = web_sys::window().unwrap().history().unwrap();

				history
					.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&href))
					.unwrap();

				event.stop_propagation();
				event.prevent_default();

				tree.dispatch(Navigate(href));
			}
		});

		let node = tree.closest_node();
		node.add_event_listener_with_callback("click", &callback.as_ref().unchecked_ref())
			.unwrap();

		tree.set_data(Rc::new(HistoryEvent(callback)));
	}

	fn diff(&self, _prev: &Self, _tree: &crate::tree::Tree<WebSys>) {
		// None
	}

	fn drop(&self, tree: &crate::tree::Tree<WebSys>, _should_unmount: bool) {
		let event = tree.remove_data::<HistoryEvent>();
		tree.closest_node()
			.remove_event_listener_with_callback("click", &event.0.as_ref().unchecked_ref())
			.unwrap();
	}
}

pub fn use_history() -> impl Markup {
	History {}
}
