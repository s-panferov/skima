pub use callback::Callback;
use wasm_bindgen::JsCast;
use web_sys::Element;

use self::helpers::cursor::Cursor;
use crate::{Backend, Markup};

pub mod attr;
pub mod bump;
pub mod callback;
pub mod context;
pub mod dispatch;
pub mod effect;
pub mod event;
pub mod helpers;
pub mod portal;
pub mod reactive;
pub mod root;
pub mod routing;
pub mod ssr;
pub mod stateful;
pub mod style;
pub mod tag;
pub mod text;

pub mod prelude {
	pub use crate::reference::reference;
	pub use crate::web::attr::{attr, classlist, classname};
	pub use crate::web::callback::Callback;
	pub use crate::web::event::on;
	pub use crate::web::html::*;
	pub use crate::web::reactive::reactive;
	pub use crate::web::style::property;
	pub use crate::web::HtmlBackend;
	pub use crate::{dynamic, Markup};
}

pub use attr::*;
pub use tag::html;

pub type Target = WebSys;

pub trait HtmlBackend: Backend {
	fn set_attribute(&self, element: &Self::Element, name: &str, value: &str);
	fn remove_attribute(&self, element: &Self::Element, name: &str);
	fn set_property(&self, element: &Self::Element, name: &str, value: &str);
	fn remove_property(&self, element: &Self::Element, name: &str);
	fn add_class(&self, element: &Self::Element, class: &str);
	fn remove_class(&self, element: &Self::Element, class: &str);
}

#[derive(Default, Clone, Debug)]
pub struct WebSys;
impl WebSys {}

impl HtmlBackend for WebSys {
	fn set_attribute(&self, node: &Self::Element, name: &str, value: &str) {
		node.set_attribute(name, value).unwrap()
	}

	fn remove_attribute(&self, node: &Self::Element, name: &str) {
		node.remove_attribute(name).unwrap()
	}

	fn set_property(&self, node: &Self::Element, name: &str, value: &str) {
		node.unchecked_ref::<web_sys::HtmlElement>()
			.style()
			.set_property(name, value)
			.unwrap();
	}

	fn remove_property(&self, node: &Self::Element, name: &str) {
		node.unchecked_ref::<web_sys::HtmlElement>()
			.style()
			.remove_property(name)
			.unwrap();
	}

	fn add_class(&self, node: &Self::Element, class: &str) {
		node.class_list().add_1(class).unwrap()
	}

	fn remove_class(&self, node: &Self::Element, class: &str) {
		node.class_list().remove_1(class).unwrap();
	}
}

impl Backend for WebSys {
	type Text = web_sys::Text;
	type Element = web_sys::Element;
	type Node = web_sys::Node;
	type Cursor = Cursor;
	type Event = web_sys::Event;

	fn replace(node: &Self::Node, prev: &Self::Node) {
		if let Some(parent_element) = prev.parent_element() {
			parent_element.replace_child(node, prev).unwrap();
		}
	}

	fn insert(cursor: Cursor, node: &Self::Node) {
		cursor.range.insert_node(node).unwrap()
	}

	fn remove(node: &Self::Node) {
		if let Some(parent_element) = node.parent_element() {
			parent_element.remove_child(node).unwrap();
		}
	}

	fn cursor_after(node: &Self::Node) -> Self::Cursor {
		Cursor::after(node).unwrap()
	}

	fn cursor_beginning_of(node: &Self::Element) -> Self::Cursor {
		Cursor::beginning_of(node).unwrap()
	}

	fn create_element(&self, tag: &'static str) -> Self::Element {
		let doc = web_sys::window().unwrap().document().unwrap();
		doc.create_element(tag).unwrap().unchecked_into()
	}

	fn create_text(&self, data: &str) -> Self::Text {
		let doc = web_sys::window().unwrap().document().unwrap();
		doc.create_text_node(data)
	}

	fn text_to_node(text: Self::Text) -> Self::Node {
		text.unchecked_into()
	}

	fn element_to_node(element: Self::Element) -> Self::Node {
		element.unchecked_into()
	}

	fn node_to_element(node: Self::Node) -> Option<Self::Element> {
		node.dyn_into::<Element>().ok()
	}

	fn set_text(&self, text: &Self::Text, data: &str) {
		text.set_text_content(Some(data));
	}

	fn node_to_text(node: Self::Node) -> Option<Self::Text> {
		node.dyn_into::<web_sys::Text>().ok()
	}
}
