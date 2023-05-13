use std::borrow::Cow;

use super::HtmlBackend;
use crate::tree::Tree;
use crate::Markup;

#[derive(Clone)]
pub struct ClassList<S: StringLike>(S);

#[derive(Clone)]
pub struct Attr<S1: StringLike, S2: StringLike>(S1, S2);
pub trait StringLike: AsRef<str> {
	const DYNAMIC: bool;
}

impl<'a> StringLike for &'a str {
	const DYNAMIC: bool = false;
}

impl StringLike for String {
	const DYNAMIC: bool = true;
}

impl<'a> StringLike for Cow<'a, str> {
	const DYNAMIC: bool = true;
}

pub fn classname<S: StringLike>(classname: S) -> Attr<&'static str, S> {
	attr("class", classname)
}

pub const fn classlist<S: StringLike>(classname: S) -> ClassList<S> {
	ClassList(classname)
}

pub fn attr<S1: StringLike, S2: StringLike>(attr: S1, value: S2) -> Attr<S1, S2> {
	Attr(attr, value)
}

impl<B: HtmlBackend, S: StringLike> Markup<B> for ClassList<S> {
	fn has_own_node() -> bool {
		false
	}

	fn dynamic() -> bool {
		S::DYNAMIC
	}

	fn render(&mut self, tree: &Tree<B>) {
		let element = B::node_to_element(tree.closest_node()).unwrap();
		tree.backend.add_class(&element, self.0.as_ref())
	}

	fn diff(&mut self, prev: &mut Self, tree: &Tree<B>) {
		if prev.0.as_ref() != self.0.as_ref() {
			let element = B::node_to_element(tree.closest_node()).unwrap();
			tree.backend.remove_class(&element, prev.0.as_ref());
			tree.backend.add_class(&element, self.0.as_ref());
		}
	}

	fn drop(&mut self, tree: &Tree<B>, should_unmount: bool) {
		if should_unmount {
			let element = B::node_to_element(tree.closest_node()).unwrap();
			tree.backend.remove_class(&element, self.0.as_ref());
		}
	}
}

impl<B: HtmlBackend, S1: StringLike, S2: StringLike> Markup<B> for Attr<S1, S2> {
	fn has_own_node() -> bool {
		false
	}

	fn dynamic() -> bool {
		S1::DYNAMIC || S2::DYNAMIC
	}

	fn render(&mut self, tree: &Tree<B>) {
		let element = B::node_to_element(tree.closest_node()).unwrap();
		tree.backend
			.set_attribute(&element, self.0.as_ref(), self.1.as_ref())
	}

	fn diff(&mut self, prev: &mut Self, tree: &Tree<B>) {
		if prev.1.as_ref() != self.1.as_ref() {
			let element = B::node_to_element(tree.closest_node()).unwrap();
			tree.backend
				.set_attribute(&element, self.0.as_ref(), self.1.as_ref());
		}
	}

	fn drop(&mut self, tree: &Tree<B>, should_unmount: bool) {
		if should_unmount {
			let element = B::node_to_element(tree.closest_node()).unwrap();
			tree.backend.remove_attribute(&element, self.0.as_ref());
		}
	}
}
