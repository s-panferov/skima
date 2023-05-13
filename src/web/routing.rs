use std::rc::Rc;

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

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

	fn render(&mut self, tree: &crate::tree::Tree<WebSys>) {
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

				event.stop_propagation();
				event.prevent_default();

				tree.dispatch(Box::new(Navigate(href)));
			}
		});

		let node = tree.closest_node();
		node.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())
			.unwrap();

		tree.data_mut().set(Rc::new(HistoryEvent(callback)));
	}

	fn diff(&mut self, _prev: &mut Self, _tree: &crate::tree::Tree<WebSys>) {
		// None
	}

	fn drop(&mut self, tree: &crate::tree::Tree<WebSys>, _should_unmount: bool) {
		let event = tree.data_mut().remove::<Rc<HistoryEvent>>();
		tree.closest_node()
			.remove_event_listener_with_callback("click", event.0.as_ref().unchecked_ref())
			.unwrap();
	}
}

pub fn use_history() -> impl Markup {
	History {}
}
