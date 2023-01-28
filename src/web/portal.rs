use wasm_bindgen::JsCast;
use web_sys::Element;

use crate::web::WebSys;
use crate::{render_subtree, subtree, Markup};

pub struct Portal<M> {
	markup: M,
	element: Element,
}

impl<M: Markup<WebSys>> Markup<WebSys> for Portal<M> {
	fn has_own_node() -> bool {
		true
	}

	fn dynamic() -> bool {
		M::dynamic()
	}

	fn render(&self, tree: &crate::tree::Tree<WebSys>) {
		tree.set_node(self.element.clone().unchecked_into());
		render_subtree(&self.markup, &tree);
	}

	fn diff(&self, prev: &Self, tree: &crate::tree::Tree<WebSys>) {
		let subtree = subtree::<M, _>(&tree);
		self.markup.diff(&prev.markup, &subtree);
	}

	fn drop(&self, tree: &crate::tree::Tree<WebSys>, should_unmount: bool) {
		let subtree = subtree::<M, _>(&tree);
		self.markup.drop(&subtree, should_unmount);
	}
}

pub fn portal<M: Markup<WebSys>>(markup: M, element: Element) -> impl Markup {
	Portal { markup, element }
}
