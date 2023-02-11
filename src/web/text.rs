use std::borrow::Cow;

use wasm_bindgen::JsCast;
use web_sys::{Node, Text};

use crate::tree::Tree;
use crate::web::helpers::dom::DOCUMENT;
use crate::web::{Markup, WebSys};

impl Markup for String {
	fn has_own_node() -> bool {
		true
	}

	fn dynamic() -> bool {
		true
	}

	fn render(&self, tree: &Tree<WebSys>) {
		tracing::debug!("Rendering text {}", self);

		let text: Node = DOCUMENT.with(|d| d.create_text_node(&self).unchecked_into());
		let prev = tree.set_node(text);
		tree.attach(prev);
	}

	fn diff(&self, prev: &Self, tree: &Tree<WebSys>) {
		tracing::debug!("Diffing text {}", self);

		if prev != self {
			tree.node()
				.unchecked_ref::<Text>()
				.set_text_content(Some(&self))
		}
	}

	fn drop(&self, tree: &Tree<WebSys>, should_unmount: bool) {
		if !should_unmount {
			return;
		}

		{
			let node = tree.node();
			let node = node.unchecked_ref::<Text>();
			node.parent_element().unwrap().remove_child(node).unwrap();
		}
	}
}

impl<'a> Markup<WebSys> for bumpalo::collections::String<'a> {
	fn has_own_node() -> bool {
		true
	}

	fn render(&self, tree: &Tree<WebSys>) {
		tracing::debug!("Rendering text {}", self);

		let text: Node = DOCUMENT.with(|d| d.create_text_node(&self).unchecked_into());
		let prev = tree.set_node(text);
		tree.attach(prev);
	}

	fn diff(&self, prev: &Self, tree: &Tree<WebSys>) {
		tracing::debug!("Diffing text {}", self);

		if prev != self {
			tree.node()
				.unchecked_ref::<Text>()
				.set_text_content(Some(&self))
		}
	}

	fn drop(&self, tree: &Tree<WebSys>, should_unmount: bool) {
		if !should_unmount {
			return;
		}

		{
			let node = tree.node();
			let node = node.unchecked_ref::<Text>();
			node.parent_element().unwrap().remove_child(node).unwrap();
		}
	}
}

impl Markup<WebSys> for &'static str {
	fn has_own_node() -> bool {
		true
	}

	fn dynamic() -> bool {
		false
	}

	fn render(&self, tree: &Tree<WebSys>) {
		tracing::debug!("Rendering text {}", self);

		let text: Node = DOCUMENT.with(|d| d.create_text_node(&self).unchecked_into());
		let prev = tree.set_node(text);
		tree.attach(prev);
	}

	fn diff(&self, prev: &Self, tree: &Tree<WebSys>) {
		tracing::debug!("Diffing text {}", self);

		if prev != self {
			tree.node()
				.unchecked_ref::<Text>()
				.set_text_content(Some(&self))
		}
	}

	fn drop(&self, tree: &Tree<WebSys>, should_unmount: bool) {
		if !should_unmount {
			return;
		}

		{
			let node = tree.node();
			let node = node.unchecked_ref::<Text>();
			node.parent_element().unwrap().remove_child(node).unwrap();
		}
	}
}

impl Markup for Cow<'static, str> {
	fn has_own_node() -> bool {
		true
	}

	fn render(&self, tree: &Tree<WebSys>) {
		tracing::debug!("Rendering text {}", self);

		let text: Node = DOCUMENT.with(|d| d.create_text_node(&self).unchecked_into());
		let prev = tree.set_node(text);
		tree.attach(prev);

		tracing::debug!("Rendering text {} done", self);
	}

	fn diff(&self, prev: &Self, tree: &Tree<WebSys>) {
		tracing::debug!("Diffing text {}", self);

		if prev != self {
			tree.node()
				.unchecked_ref::<Text>()
				.set_text_content(Some(&self))
		}
	}

	fn drop(&self, tree: &Tree<WebSys>, should_unmount: bool) {
		if !should_unmount {
			return;
		}

		{
			let node = tree.node();
			let node = node.unchecked_ref::<Text>();
			node.parent_element().unwrap().remove_child(node).unwrap();
		}
	}
}
