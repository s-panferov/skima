use super::{HtmlBackend, StringLike};
use crate::tree::Tree;
use crate::Markup;
pub struct Property<S1: StringLike, S2: StringLike>(S1, S2);

pub fn property<S1: StringLike, S2: StringLike>(prop: S1, value: S2) -> Property<S1, S2> {
	Property(prop, value)
}

impl<B: HtmlBackend, S1: StringLike, S2: StringLike> Markup<B> for Property<S1, S2> {
	fn has_own_node() -> bool {
		false
	}

	fn dynamic() -> bool {
		S1::DYNAMIC || S2::DYNAMIC
	}

	fn render(&self, tree: &Tree<B>) {
		let element = B::node_to_element(tree.closest_node()).unwrap();
		tree.backend
			.set_property(&element, self.0.as_ref(), self.1.as_ref());
	}

	fn diff(&self, prev: &Self, tree: &Tree<B>) {
		if prev.1.as_ref() != self.1.as_ref() {
			let element = B::node_to_element(tree.closest_node()).unwrap();
			tree.backend
				.set_property(&element, self.0.as_ref(), self.1.as_ref());
		}
	}

	fn drop(&self, tree: &Tree<B>, should_unmount: bool) {
		if should_unmount {
			let element = B::node_to_element(tree.closest_node()).unwrap();
			tree.backend.remove_property(&element, self.0.as_ref());
		}
	}
}
