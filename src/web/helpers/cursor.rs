use wasm_bindgen::JsValue;
use web_sys::{Node, Range};

#[allow(unused)]
pub struct Cursor {
	pub range: Range,
}

impl Cursor {
	pub fn before(node: &Node) -> Result<Self, JsValue> {
		let document = node.owner_document().expect("Node should be attached");
		let range = document.create_range()?;
		range.set_start_before(&node)?;
		range.set_end_before(&node)?;
		Ok(Cursor { range })
	}

	pub fn after(node: &Node) -> Result<Self, JsValue> {
		let document = node.owner_document().expect("Node should be attached");
		let range = document.create_range()?;
		range.set_start_after(&node)?;
		range.set_end_after(&node)?;
		Ok(Cursor { range })
	}

	pub fn beginning_of(node: &Node) -> Result<Self, JsValue> {
		let document = node.owner_document().expect("Node should be attached");
		let range = document.create_range()?;
		range.set_start(&node, 0)?;
		range.set_end(&node, 0)?;
		Ok(Cursor { range })
	}
}
