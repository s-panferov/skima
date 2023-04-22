use std::borrow::Borrow;
use std::cell::RefCell;

use crate::{Backend, HtmlBackend};

#[derive(Clone, Debug)]
pub struct StaticHtml<'a> {
	pub bump: &'a bumpalo::Bump,
}

#[derive(Debug, Clone)]
pub enum StaticNode<'a> {
	Element(&'a StaticElement<'a>),
	Text(&'a StaticText<'a>),
}

#[derive(Debug)]
pub struct StaticElement<'a> {
	pub tag: &'a str,
	pub parent: RefCell<Option<&'a StaticElement<'a>>>,
	pub children: RefCell<Vec<StaticNode<'a>>>,
}

impl<'a> StaticElement<'a> {
	pub fn new(tag: &'a str) -> Self {
		StaticElement {
			tag,
			parent: RefCell::new(None),
			children: RefCell::new(Vec::new()),
		}
	}
}

impl<'a> StaticElement<'a> {
	pub fn to_html(&self, buffer: &mut String) -> Result<(), std::fmt::Error> {
		use std::fmt::Write;
		write!(buffer, "<{}>", self.tag)?;
		let children = self.children.borrow();
		for child in children.iter() {
			match child {
				StaticNode::Element(node) => {
					node.to_html(buffer)?;
				}
				StaticNode::Text(text) => {
					write!(buffer, "{}", text.text.borrow())?;
				}
			}
		}
		write!(buffer, "</{}>", self.tag)?;
		Ok(())
	}
}

#[derive(Debug)]
pub struct StaticText<'a> {
	pub parent: RefCell<Option<&'a StaticElement<'a>>>,
	pub text: RefCell<&'a str>,
}

impl<'a> StaticText<'a> {
	pub fn new(text: &'a str) -> Self {
		StaticText {
			parent: RefCell::new(None),
			text: RefCell::new(text),
		}
	}
}

pub enum StaticCursor<'a> {
	After(StaticNode<'a>),
	BeginningOf(&'a StaticElement<'a>),
}

impl<'a> StaticNode<'a> {
	pub fn parent(&self) -> Option<&'a StaticElement<'a>> {
		match self {
			Self::Element(node) => node.parent.borrow().clone(),
			Self::Text(text) => text.parent.borrow().clone(),
		}
	}

	pub fn set_parent(
		&self,
		parent: Option<&'a StaticElement<'a>>,
	) -> Option<&'a StaticElement<'a>> {
		match self {
			Self::Element(node) => node.parent.replace(parent),
			Self::Text(text) => text.parent.replace(parent),
		}
	}
}

impl<'a> HtmlBackend for StaticHtml<'a> {}

impl<'a> Backend for StaticHtml<'a> {
	type Element = &'a StaticElement<'a>;
	type Text = &'a StaticText<'a>;
	type Node = StaticNode<'a>;
	type Cursor = StaticCursor<'a>;
	type Event = ();

	fn replace(node: &Self::Node, prev: &Self::Node) {
		let parent = prev.parent().unwrap();
		let index = find_index(parent, prev.clone());
		node.set_parent(Some(parent));
		parent.children.borrow_mut()[index] = node.clone();
	}

	fn remove(node: &Self::Node) {
		let parent = node.parent().clone().unwrap();
		let index = find_index(parent, node.clone());
		node.set_parent(None);
		parent.children.borrow_mut().remove(index);
	}

	fn insert(cursor: StaticCursor<'a>, node: &Self::Node) {
		match cursor {
			StaticCursor::After(after) => {
				let parent = after.parent().unwrap();
				let index = find_index(parent, after);
				node.set_parent(Some(parent));
				parent.children.borrow_mut().insert(index + 1, node.clone());
			}
			StaticCursor::BeginningOf(parent) => {
				node.set_parent(Some(parent));
				parent.children.borrow_mut().insert(0, node.clone());
			}
		}
		// cursor.range.insert_node(node).unwrap()
	}

	fn cursor_after(node: &Self::Node) -> Self::Cursor {
		StaticCursor::After(node.clone())
	}

	fn cursor_beginning_of(node: &Self::Element) -> Self::Cursor {
		StaticCursor::BeginningOf(node)
	}

	fn create_element(&self, tag: &'static str) -> Self::Element {
		self.bump.alloc_with(|| StaticElement::new(tag))
	}

	fn create_text(&self, data: &str) -> Self::Text {
		let data = self.bump.alloc_str(data);
		self.bump.alloc_with(|| StaticText::new(data))
	}

	fn text_to_node(text: Self::Text) -> Self::Node {
		StaticNode::Text(text)
	}

	fn element_to_node(element: Self::Element) -> Self::Node {
		StaticNode::Element(element)
	}

	fn node_to_element(node: Self::Node) -> Option<Self::Element> {
		match node {
			StaticNode::Element(el) => Some(el),
			_ => None,
		}
	}

	type Phantom = std::marker::PhantomData<Self>;

	fn set_text(&self, text: &Self::Text, data: &str) {
		let data = self.bump.alloc_str(data);
		text.text.replace(data);
	}

	fn node_to_text(node: Self::Node) -> Option<Self::Text> {
		todo!()
	}
}

fn find_index<'a>(parent: &'a StaticElement<'a>, node: StaticNode<'a>) -> usize {
	parent
		.children
		.borrow()
		.iter()
		.enumerate()
		.find(|(_, el)| match (&node, *el) {
			(StaticNode::Element(a), StaticNode::Element(b)) => std::ptr::eq(*a, *b),
			(StaticNode::Text(a), StaticNode::Text(b)) => std::ptr::eq(*a, *b),
			_ => false,
		})
		.unwrap()
		.0
}
