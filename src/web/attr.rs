use std::borrow::Cow;

use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::tree::Tree;
use crate::web::WebSys;
use crate::Markup;

#[derive(Clone)]
pub struct ClassList<S: StringLike>(S);

#[derive(Clone)]
pub struct Attr<S1: StringLike, S2: StringLike>(S1, S2);

pub trait StringLike: AsRef<str> {
	const DYNAMIC: bool;
}

impl StringLike for &'static str {
	const DYNAMIC: bool = false;
}

impl StringLike for String {
	const DYNAMIC: bool = true;
}

impl<'a> StringLike for Cow<'a, str> {
	const DYNAMIC: bool = false;
}

pub fn classname<S: StringLike>(classname: S) -> Attr<&'static str, S> {
	attr("class", classname)
}

pub fn classlist<S: StringLike>(classname: S) -> ClassList<S> {
	ClassList(classname)
}

pub fn attr<S1: StringLike, S2: StringLike>(attr: S1, value: S2) -> Attr<S1, S2> {
	Attr(attr.into(), value.into())
}

impl<S: StringLike> Markup for ClassList<S> {
	fn has_own_node() -> bool {
		false
	}

	fn dynamic() -> bool {
		S::DYNAMIC
	}

	fn render(&self, tree: &Tree<WebSys>) {
		tree.closest_node()
			.unchecked_ref::<HtmlElement>()
			.class_list()
			.add_1(self.0.as_ref())
			.unwrap();
	}

	fn diff(&self, prev: &Self, tree: &Tree<WebSys>) {
		if !Self::dynamic() {
			return;
		}

		if prev.0.as_ref() != self.0.as_ref() {
			let element = tree.closest_node();
			let element = element.unchecked_ref::<HtmlElement>();
			element.class_list().remove_1(&prev.0.as_ref()).unwrap();
			element.class_list().add_1(&self.0.as_ref()).unwrap();
		}
	}

	fn drop(&self, tree: &Tree<WebSys>, should_unmount: bool) {
		if should_unmount {
			tree.closest_node()
				.unchecked_ref::<HtmlElement>()
				.class_list()
				.remove_1(&self.0.as_ref())
				.unwrap();
		}
	}
}

impl<S1: StringLike, S2: StringLike> Markup for Attr<S1, S2> {
	fn has_own_node() -> bool {
		false
	}

	fn dynamic() -> bool {
		S1::DYNAMIC && S2::DYNAMIC
	}

	fn render(&self, tree: &Tree<WebSys>) {
		tree.closest_node()
			.unchecked_ref::<HtmlElement>()
			.set_attribute(&self.0.as_ref(), &self.1.as_ref())
			.unwrap();
	}

	fn diff(&self, prev: &Self, tree: &Tree<WebSys>) {
		if !Self::dynamic() {
			return;
		}

		if prev.0.as_ref() != self.0.as_ref() {
			tree.closest_node()
				.unchecked_ref::<HtmlElement>()
				.set_attribute(&self.0.as_ref(), &self.1.as_ref())
				.unwrap();
		}
	}

	fn drop(&self, tree: &Tree<WebSys>, should_unmount: bool) {
		if should_unmount {
			tree.closest_node()
				.unchecked_ref::<HtmlElement>()
				.remove_attribute(&self.0.as_ref())
				.unwrap();
		}
	}
}
