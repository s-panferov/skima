use std::any::Any;
use std::rc::Rc;

use super::Callback;
use crate::action::Action;
use crate::anydata::Envelope;
use crate::tree::Tree;
use crate::web::{Markup, WebSys};
use crate::Backend;

pub enum ActionResult {
	Propagate(Box<dyn Action>),
	Stop,
}

impl Default for ActionResult {
	fn default() -> Self {
		Self::Stop
	}
}

pub type ActionHandler = Callback<dyn Fn(Box<dyn Action>) -> ActionResult>;

pub struct Dispatcher {
	func: Rc<dyn Fn(Box<dyn Action>)>,
}

// FIXME: avoid double boxing somehow
impl Envelope for Dispatcher {
	type Output = Dispatcher;

	fn from_dyn(rc: Rc<dyn Any>) -> Self::Output {
		let boxed = Rc::downcast::<Dispatcher>(rc).map_err(|_| ()).unwrap();
		(&*boxed).clone()
	}

	fn to_dyn(self) -> Rc<dyn Any> {
		Rc::new(self)
	}
}

impl Clone for Dispatcher {
	fn clone(&self) -> Self {
		Self {
			func: self.func.clone(),
		}
	}
}

impl Dispatcher {
	pub fn dispatch<T: Action>(&self, op: T) {
		(self.func)(Box::new(op) as Box<dyn Action>)
	}

	pub fn dispatch_dyn(&self, op: Box<dyn Action>) {
		(self.func)(op)
	}

	pub fn new<F>(func: F) -> Self
	where
		F: Fn(Box<dyn Action>) + 'static,
	{
		Dispatcher {
			func: Rc::new(func),
		}
	}
}

struct Provide<T, M> {
	data: Rc<T>,
	markup: M,
}

impl<T: Any + 'static, M: Markup<WebSys>> Markup<WebSys> for Provide<T, M> {
	fn has_own_node() -> bool {
		M::has_own_node()
	}

	fn render(&self, tree: &crate::tree::Tree<WebSys>) {
		tree.data_mut().set(self.data.clone());
		self.markup.render(tree);
	}

	fn diff(&self, prev: &Self, tree: &crate::tree::Tree<WebSys>) {
		self.markup.diff(&prev.markup, &tree);
	}

	fn drop(&self, tree: &crate::tree::Tree<WebSys>, should_unmount: bool) {
		tree.data_mut().remove::<Rc<T>>();
		self.markup.drop(tree, should_unmount)
	}
}

pub fn provide<T: Any + 'static>(value: T, markup: impl Markup) -> impl Markup {
	Provide {
		data: Rc::new(value),
		markup,
	}
}

pub struct Capture<M: Markup<B>, B: Backend> {
	markup: M,
	handler: ActionHandler,
	_b: B::Phantom,
}

pub fn capture(markup: impl Markup, handler: ActionHandler) -> impl Markup {
	Capture {
		markup,
		handler,
		_b: Default::default(),
	}
}

impl<M: Markup<B>, B: Backend> Markup<B> for Capture<M, B> {
	fn has_own_node() -> bool {
		M::has_own_node()
	}

	fn render(&self, tree: &Tree<B>) {
		tree.capture
			.borrow_mut()
			.insert(fxhash::hash64(&self.handler), self.handler.clone());

		self.markup.render(tree);
	}

	fn diff(&self, prev: &Self, tree: &Tree<B>) {
		self.markup.diff(&prev.markup, tree);
	}

	fn drop(&self, tree: &Tree<B>, should_unmount: bool) {
		tree.capture
			.borrow_mut()
			.remove(&fxhash::hash64(&self.handler));

		self.markup.drop(tree, should_unmount)
	}
}
