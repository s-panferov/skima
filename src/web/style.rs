use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use super::{StringLike, WebSys};
use crate::tree::Tree;
use crate::Markup;
pub struct Property<S1: StringLike, S2: StringLike>(S1, S2);

pub fn property<S1: StringLike, S2: StringLike>(prop: S1, value: S2) -> Property<S1, S2> {
	Property(prop, value)
}

impl<S1: StringLike, S2: StringLike> Markup for Property<S1, S2> {
	fn has_own_node() -> bool {
		false
	}

	fn dynamic() -> bool {
		S1::DYNAMIC || S2::DYNAMIC
	}

	fn render(&self, tree: &Tree<WebSys>) {
		tree.closest_node()
			.unchecked_ref::<HtmlElement>()
			.style()
			.set_property(self.0.as_ref(), self.1.as_ref())
			.unwrap();
	}

	fn diff(&self, prev: &Self, tree: &Tree<WebSys>) {
		if prev.1.as_ref() != self.1.as_ref() {
			tree.closest_node()
				.unchecked_ref::<HtmlElement>()
				.style()
				.set_property(self.0.as_ref(), self.1.as_ref())
				.unwrap();
		}
	}

	fn drop(&self, tree: &Tree<WebSys>, should_unmount: bool) {
		if should_unmount {
			tree.closest_node()
				.unchecked_ref::<HtmlElement>()
				.style()
				.remove_property(self.0.as_ref())
				.unwrap();
		}
	}
}
