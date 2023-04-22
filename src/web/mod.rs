pub use callback::Callback;

use self::helpers::cursor::Cursor;
use crate::{Backend, Markup};

pub mod attr;
pub mod bump;
pub mod callback;
pub mod constant;
pub mod dispatch;
pub mod effect;
pub mod event;
pub mod helpers;
mod hook;
pub mod portal;
pub mod reactive;
pub mod root;
pub mod routing;
pub mod style;
pub mod tag;
pub mod text;

pub use hook::hook;

pub mod prelude {
	pub use crate::reference::reference;
	pub use crate::web::attr::{attr, classlist, classname};
	pub use crate::web::callback::Callback;
	pub use crate::web::event::on;
	pub use crate::web::html::*;
	pub use crate::web::reactive::reactive;
	pub use crate::web::style::property;
	pub use crate::{dynamic, Markup};
}

pub use attr::*;
pub use tag::html;

pub type Target = WebSys;

#[derive(Default, Debug)]
pub struct WebSys {}
impl WebSys {}

impl Backend for WebSys {
	type Node = web_sys::Node;
	type Cursor = Cursor;
	type Event = web_sys::Event;
	type Data = ();

	fn replace(node: &Self::Node, prev: &Self::Node) {
		if let Some(parent_element) = prev.parent_element() {
			parent_element.replace_child(node, prev).unwrap();
		}
	}

	fn insert(cursor: Cursor, node: &Self::Node) {
		cursor.range.insert_node(node).unwrap()
	}

	fn cursor_after(node: &Self::Node) -> Self::Cursor {
		Cursor::after(node).unwrap()
	}

	fn cursor_beginning_of(node: &Self::Node) -> Self::Cursor {
		Cursor::beginning_of(node).unwrap()
	}
}
