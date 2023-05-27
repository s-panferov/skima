use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Deref;
use std::rc::Rc;

use by_address::ByAddress;
use indexmap::IndexSet;

use crate::action::Action;
use crate::anydata::AnyData;
use crate::web::dispatch::{ActionHandler, ActionResult};
use crate::Backend;

pub struct Tree<B: Backend>(Rc<TreeInner<B>>);

impl<B: Backend> PartialEq for Tree<B> {
	fn eq(&self, other: &Self) -> bool {
		Rc::ptr_eq(&self.0, &other.0)
	}
}

impl<B: Backend> Eq for Tree<B> {}

impl<B: Backend> Clone for Tree<B> {
	fn clone(&self) -> Self {
		Tree(self.0.clone())
	}
}

impl<B: Backend> Deref for Tree<B> {
	type Target = Rc<TreeInner<B>>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

pub struct TreeInner<B: Backend> {
	pub(crate) level: usize,

	pub(crate) parent: Option<Tree<B>>,
	pub(crate) prev: RefCell<Option<Tree<B>>>,
	pub(crate) next: RefCell<Option<Tree<B>>>,

	// TODO: Optimize for a single child?
	pub(crate) children: RefCell<IndexSet<ByAddress<Tree<B>>>>,

	// HTML node
	pub(crate) node: RefCell<Option<B::Node>>,

	// Capture state
	pub(crate) capture: RefCell<HashMap<u64, ActionHandler>>,

	// Mutable element state
	pub(crate) data: RefCell<AnyData>,

	pub(crate) backend: B,
}

impl<B: Backend> Debug for Tree<B>
where
	B: Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct(&format!("Tree[{:?}]", Rc::as_ptr(&self.0)))
			.field("parent", &self.parent.as_ref().map(|p| Rc::as_ptr(&p.0)))
			.field(
				"prev",
				&self.prev.borrow().as_ref().map(|p| Rc::as_ptr(&p.0)),
			)
			.field(
				"next",
				&self.next.borrow().as_ref().map(|p| Rc::as_ptr(&p.0)),
			)
			.field("children", &self.children.borrow())
			.field("node", &self.node)
			.finish()
	}
}

impl<B: Backend> Tree<B> {
	pub fn root(node: B::Element, data: B) -> Self {
		Tree(Rc::new(TreeInner {
			level: 0,
			parent: None,
			capture: Default::default(),
			prev: RefCell::new(None),
			next: RefCell::new(None),
			children: RefCell::new(IndexSet::new()),
			node: RefCell::new(Some(B::element_to_node(node))),
			data: RefCell::new(Default::default()),
			backend: data,
		}))
	}

	pub fn ephemeral_root(backend: B) -> Self {
		Tree(Rc::new(TreeInner {
			level: 0,
			parent: None,
			capture: Default::default(),
			prev: RefCell::new(None),
			next: RefCell::new(None),
			children: RefCell::new(IndexSet::new()),
			node: RefCell::new(None),
			data: RefCell::new(Default::default()),
			backend: backend,
		}))
	}

	pub fn new(parent: &Tree<B>) -> Self {
		let fiber = Tree(Rc::new(TreeInner {
			level: parent.level + 1,
			capture: Default::default(),
			parent: Some(parent.clone()),
			prev: RefCell::new(None),
			next: RefCell::new(None),
			children: RefCell::new(IndexSet::new()),
			node: RefCell::new(None),
			data: RefCell::new(Default::default()),
			backend: parent.backend.clone(),
		}));

		if let Some(prev) = parent.children.borrow().last() {
			fiber.prev.replace(Some(prev.0.clone()));
			prev.next.replace(Some(fiber.clone()));
		}

		parent
			.children
			.borrow_mut()
			.insert(ByAddress(fiber.clone()));

		fiber
	}

	pub fn insert_at(&self, index: usize) -> Self {
		let tree = Tree(Rc::new(TreeInner {
			level: self.level + 1,
			parent: Some(self.clone()),
			prev: RefCell::new(None),
			next: RefCell::new(None),
			capture: Default::default(),
			children: RefCell::new(IndexSet::new()),
			node: RefCell::new(None),
			data: RefCell::new(Default::default()),
			backend: self.backend.clone(),
		}));

		if index > self.children.borrow().len() {
			panic!()
		}

		if let Some(child) = self.children.borrow().get_index(index) {
			if let Some(prev) = child.prev.borrow().clone() {
				prev.next.replace(Some(tree.clone()));
				tree.prev.replace(Some(prev));
			}
			tree.next.replace(Some(child.0.clone()));
			child.prev.replace(Some(tree.clone()));
		} else if let Some(prev) = self.children.borrow().last() {
			prev.next.replace(Some(tree.clone()));
			tree.prev.replace(Some(prev.0.clone()));
		}

		let (i, _) = self
			.children
			.borrow_mut()
			.insert_full(ByAddress(tree.clone()));

		self.children.borrow_mut().move_index(i, index);

		tree
	}

	pub fn remove_at(&self, index: usize) {
		let element = self
			.children
			.borrow_mut()
			.shift_remove_index(index)
			.unwrap();

		element.disconnect(true);
	}

	pub fn first_child(&self) -> Tree<B> {
		self.children.borrow().first().as_ref().unwrap().0.clone()
	}

	pub fn data(&self) -> Ref<AnyData> {
		self.data.borrow()
	}

	pub fn data_mut(&self) -> RefMut<AnyData> {
		self.data.borrow_mut()
	}

	pub fn dispatch(&self, action: Box<dyn Action>) {
		let mut cursor = { Some(self.clone()) };

		let mut action = Some(action);
		while let Some(tree) = cursor {
			for item in tree.capture.borrow().values() {
				match item(action.take().unwrap()) {
					ActionResult::Propagate(a) => action = Some(a),
					ActionResult::Stop => return,
				}
			}
			cursor = tree.parent.clone();
		}
	}

	pub fn closest_node(&self) -> B::Node {
		if let Some(node) = self.node.borrow().as_ref() {
			node.clone()
		} else {
			self.parent.as_ref().unwrap().closest_node()
		}
	}

	pub fn child_at(&self, index: usize) -> Tree<B> {
		self.children
			.borrow()
			.get_index(index)
			.as_ref()
			.unwrap()
			.0
			.clone()
	}

	pub fn node(&self) -> Ref<'_, B::Node> {
		Ref::map(self.node.borrow(), |v: &Option<B::Node>| {
			v.as_ref().unwrap()
		})
	}

	pub fn next(&self) -> Tree<B> {
		self.next.borrow().clone().unwrap()
	}

	pub fn prev(&self) -> Tree<B> {
		self.prev.borrow().clone().unwrap()
	}

	pub fn clear(&self) {
		{
			for child in self.children.borrow().iter() {
				child.disconnect(false)
			}
		}

		// FIXME: If we are going to enable this
		//        ReactiveComponents should always create
		//        an own tree node.

		// self.data.borrow_mut().clear();
		self.children.borrow_mut().clear();
	}

	// Break all the cycles
	pub fn disconnect(&self, fix_siblings: bool) {
		// PARENT WILL PROBABLY REMOVE US ANYWAY

		// if let Some(parent) = &self.parent {
		// 	parent
		// 		.children
		// 		.borrow_mut()
		// 		.remove(&ByAddress(self.clone()));
		// }

		if fix_siblings {
			match (&*self.prev.borrow(), &*self.next.borrow()) {
				(Some(prev), Some(next)) => {
					prev.next.replace(Some(next.clone()));
					next.prev.replace(Some(prev.clone()));
				}
				(Some(prev), None) => {
					prev.next.replace(None);
				}
				(None, Some(next)) => {
					next.prev.replace(None);
				}
				(None, None) => {}
			}
		}

		self.prev.replace(None);
		self.next.replace(None);
		self.clear()
	}

	pub fn set_node(&self, node: B::Node) -> Option<B::Node> {
		self.node.replace(Some(node))
	}

	pub fn remove_node(&self) -> Option<B::Node> {
		self.node.replace(None)
	}

	pub fn unmount(&self) {
		B::remove(&self.node());
		self.node.replace(None);
	}

	pub fn attach(&self, prev: Option<B::Node>) {
		let node = self.node.borrow();
		let node = node.as_ref().unwrap();
		if let Some(prev) = prev {
			B::replace(node, &prev)
		} else if let Some(cursor) = self.find_pacement() {
			B::insert(cursor, node);
		}
	}

	pub fn fist_node(&self) -> Option<B::Node> {
		if let Some(node) = self.node.borrow().as_ref() {
			return Some(node.clone());
		}

		for child in self.children.borrow().iter() {
			if let Some(node) = child.fist_node() {
				return Some(node);
			}
		}

		None
	}

	pub fn last_node(&self) -> Option<B::Node> {
		if let Some(node) = self.node.borrow().as_ref() {
			return Some(node.clone());
		}

		for child in self.children.borrow().iter().rev() {
			if let Some(node) = child.last_node() {
				return Some(node);
			}
		}

		None
	}

	pub fn find_pacement(&self) -> Option<B::Cursor> {
		let mut cursor = self.clone();

		loop {
			while cursor.prev.borrow().is_none() {
				if let Some(parent) = &cursor.parent {
					let node = parent.node.borrow();
					if let Some(node) = &*node {
						return Some(B::cursor_beginning_of(
							&B::node_to_element(node.clone()).unwrap(),
						));
					} else {
						std::mem::drop(node);
						cursor = parent.clone()
					};
				} else {
					return None;
				}
			}

			let prev_b = cursor.prev.borrow();
			let prev = prev_b.clone().unwrap();
			std::mem::drop(prev_b);
			cursor = prev;

			if let Some(node) = cursor.last_node() {
				return Some(B::cursor_after(&node));
			}
		}
	}
}

/*
(
	(a, a, a)
	(b)
)
*/

#[derive(Debug, Clone)]
struct Noop;

impl Backend for Noop {
	type Cursor = ();
	type Event = ();
	type Element = String;

	fn cursor_after(_node: &Self::Node) -> Self::Cursor {}

	fn cursor_beginning_of(_node: &Self::Element) -> Self::Cursor {}

	fn insert(_cursor: Self::Cursor, _node: &Self::Node) {}

	fn replace(_node: &Self::Node, _prev: &Self::Node) {}

	type Text = String;

	fn create_element(&self, tag: &'static str) -> Self::Element {
		tag.into()
	}

	fn create_text(&self, data: &str) -> Self::Text {
		data.to_owned()
	}

	fn remove(node: &Self::Node) {}

	fn text_to_node(text: Self::Text) -> Self::Node {
		text
	}

	fn element_to_node(element: Self::Element) -> Self::Node {
		element
	}

	fn node_to_element(node: Self::Node) -> Option<Self::Element> {
		Some("".into())
	}

	type Node = String;

	fn set_text(&self, text: &Self::Text, data: &str) {}

	fn node_to_text(node: Self::Node) -> Option<Self::Text> {
		None
	}
}

#[cfg(test)]
mod tests {
	use std::borrow::Borrow;

	use super::*;

	#[test]
	fn test() {
		let root = Tree::<Noop>::root("Root".into(), Noop);
		let child1 = Tree::new(&root);
		child1.set_node("Child 1".into());

		assert_eq!(root.children.borrow().len(), 1);
		assert_eq!(*child1.parent.borrow().as_ref().unwrap(), root);

		let child2 = Tree::new(&root);
		child1.set_node("Child 2".into());

		assert_eq!(root.children.borrow().len(), 2);
		assert_eq!(child1.next(), child2);
		assert_eq!(child2.prev(), child1);

		// Insert in the middle
		let child3 = root.insert_at(1);
		child1.set_node("Child 3".into());

		assert_eq!(root.children.borrow().len(), 3);

		assert_eq!(**root.children.borrow().get_index(0).unwrap(), child1);
		assert_eq!(**root.children.borrow().get_index(1).unwrap(), child3);
		assert_eq!(**root.children.borrow().get_index(2).unwrap(), child2);

		assert_eq!(child1.next(), child3);
		assert_eq!(child3.next(), child2);
		assert_eq!(*child2.next.borrow(), None);

		assert_eq!(child2.prev(), child3);
		assert_eq!(child3.prev(), child1);
		assert_eq!(*child1.prev.borrow(), None);

		root.remove_at(1);

		assert_eq!(root.children.borrow().len(), 2);

		assert_eq!(**root.children.borrow().get_index(0).unwrap(), child1);
		assert_eq!(**root.children.borrow().get_index(1).unwrap(), child2);

		assert_eq!(child1.next(), child2);
		assert_eq!(child2.prev(), child1);

		assert_eq!(*child1.prev.borrow(), None);
		assert_eq!(*child2.next.borrow(), None);

		assert_eq!(*child3.prev.borrow(), None);
		assert_eq!(*child3.next.borrow(), None);
	}
}
