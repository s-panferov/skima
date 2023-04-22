use std::cell::RefCell;

use crate::Backend;

#[derive(Clone, Debug)]
pub struct StaticHtml<'a> {
	pub bump: &'a bumpalo::Bump,
}

#[derive(Debug)]
pub enum StaticNodeOrText<'a> {
	Node(&'a StaticNode<'a>),
	Text(&'a StaticText<'a>),
}

#[derive(Debug)]
pub struct StaticNode<'a> {
	pub tag: &'a str,
	pub parent: RefCell<Option<&'a StaticNode<'a>>>,
	pub children: RefCell<Vec<StaticNodeOrText<'a>>>,
}

impl<'a> StaticNode<'a> {
	pub fn to_html(&self, buffer: &mut String) -> Result<(), std::fmt::Error> {
		use std::fmt::Write;
		write!(buffer, "<{}>", self.tag)?;
		let children = self.children.borrow();
		for child in children.iter() {
			match child {
				StaticNodeOrText::Node(node) => {
					node.to_html(buffer)?;
				}
				StaticNodeOrText::Text(text) => {
					write!(buffer, "{}", text.text)?;
				}
			}
		}
		write!(buffer, "</{}>", self.tag)?;
		Ok(())
	}
}

#[derive(Debug)]
pub struct StaticText<'a> {
	pub parent: RefCell<Option<&'a StaticNode<'a>>>,
	pub text: bumpalo::collections::String<'a>,
}

pub enum StaticCursor<'a> {
	After(StaticNodeOrText<'a>),
	BeginningOf(&'a StaticNode<'a>),
}

impl<'a> StaticNodeOrText<'a> {
	pub fn parent(&self) -> Option<&'a StaticNode<'a>> {
		match self {
			Self::Node(node) => node.parent.borrow().clone(),
			Self::Text(text) => text.parent.borrow().clone(),
		}
	}
}

impl<'a> Backend for StaticHtml<'a> {
	type Node = &'a StaticNode<'a>;
	type Cursor = StaticCursor<'a>;
	type Event = ();
	type Data = Self;

	fn replace(node: &Self::Node, prev: &Self::Node) {
		let parent = prev.parent.borrow().clone().unwrap();
		let index = find_index(parent, StaticNodeOrText::Node(prev));
		node.parent.replace(prev.parent.borrow().clone());
		parent.children.borrow_mut()[index] = StaticNodeOrText::Node(node);
	}

	fn insert(cursor: StaticCursor<'a>, node: &Self::Node) {
		match cursor {
			StaticCursor::After(after) => {
				let parent = after.parent().unwrap();
				let index = find_index(parent, after);
				node.parent.replace(Some(parent));
				parent
					.children
					.borrow_mut()
					.insert(index + 1, StaticNodeOrText::Node(node));
			}
			StaticCursor::BeginningOf(parent) => {
				node.parent.replace(Some(parent));
				parent
					.children
					.borrow_mut()
					.insert(0, StaticNodeOrText::Node(node));
			}
		}
		// cursor.range.insert_node(node).unwrap()
	}

	fn cursor_after(node: &Self::Node) -> Self::Cursor {
		StaticCursor::After(StaticNodeOrText::Node(node))
	}

	fn cursor_beginning_of(node: &Self::Node) -> Self::Cursor {
		StaticCursor::BeginningOf(node)
	}
}

fn find_index<'a>(parent: &'a StaticNode<'a>, node: StaticNodeOrText<'a>) -> usize {
	parent
		.children
		.borrow()
		.iter()
		.enumerate()
		.find(|(_, el)| match (&node, *el) {
			(StaticNodeOrText::Node(a), StaticNodeOrText::Node(b)) => std::ptr::eq(*a, *b),
			(StaticNodeOrText::Text(a), StaticNodeOrText::Text(b)) => std::ptr::eq(*a, *b),
			_ => false,
		})
		.unwrap()
		.0
}
