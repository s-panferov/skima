use std::borrow::Cow;

use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::tree::Tree;
use crate::web::WebSys;
use crate::Markup;

#[derive(Clone)]
pub struct ClassList(Cow<'static, str>);

#[derive(Clone)]
pub struct Attr(Cow<'static, str>, Cow<'static, str>);

pub fn classname(classname: impl Into<Cow<'static, str>>) -> Attr {
	attr("class", classname)
}

pub fn classlist(classname: impl Into<Cow<'static, str>>) -> ClassList {
	ClassList(classname.into())
}

pub fn attr(attr: impl Into<Cow<'static, str>>, value: impl Into<Cow<'static, str>>) -> Attr {
	Attr(attr.into(), value.into())
}

impl Markup for ClassList {
	fn has_own_node() -> bool {
		false
	}

	fn render(&self, tree: &Tree<WebSys>) {
		tree.closest_node()
			.unchecked_ref::<HtmlElement>()
			.class_list()
			.add_1(&self.0)
			.unwrap();
	}

	fn diff(&self, prev: &Self, tree: &Tree<WebSys>) {
		if prev.0 != self.0 {
			let element = tree.closest_node();
			let element = element.unchecked_ref::<HtmlElement>();
			element.class_list().remove_1(&prev.0).unwrap();
			element.class_list().add_1(&self.0).unwrap();
		}
	}

	fn drop(&self, tree: &Tree<WebSys>, should_unmount: bool) {
		if should_unmount {
			tree.closest_node()
				.unchecked_ref::<HtmlElement>()
				.class_list()
				.remove_1(&self.0)
				.unwrap();
		}
	}
}

impl Markup for Attr {
	fn has_own_node() -> bool {
		false
	}

	fn render(&self, tree: &Tree<WebSys>) {
		tree.closest_node()
			.unchecked_ref::<HtmlElement>()
			.set_attribute(&self.0, &self.1)
			.unwrap();
	}

	fn diff(&self, prev: &Self, tree: &Tree<WebSys>) {
		if prev.0 != self.0 {
			tree.closest_node()
				.unchecked_ref::<HtmlElement>()
				.set_attribute(&self.0, &self.1)
				.unwrap();
		}
	}

	fn drop(&self, tree: &Tree<WebSys>, should_unmount: bool) {
		if should_unmount {
			tree.closest_node()
				.unchecked_ref::<HtmlElement>()
				.remove_attribute(&self.0)
				.unwrap();
		}
	}
}
