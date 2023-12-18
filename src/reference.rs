use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::anydata::Envelope;
use crate::{Backend, Markup};

pub struct Mutable<T> {
	inner: Rc<RefCell<T>>,
}

impl<T: 'static> Envelope for Mutable<T> {
	type Output = Mutable<T>;

	fn to_dyn(self) -> Rc<dyn std::any::Any> {
		self.inner
	}

	fn from_dyn(rc: Rc<dyn std::any::Any>) -> Self::Output {
		Mutable {
			inner: rc.downcast().map_err(|_| ()).unwrap(),
		}
	}
}

impl<T: Default> Default for Mutable<T> {
	fn default() -> Self {
		Mutable::new(Default::default())
	}
}

impl<T> Clone for Mutable<T> {
	fn clone(&self) -> Self {
		Mutable {
			inner: self.inner.clone(),
		}
	}
}

impl<T> Mutable<T> {
	pub fn new(value: T) -> Mutable<T> {
		Mutable {
			inner: Rc::new(RefCell::new(value)),
		}
	}

	pub fn set(&self, value: T) {
		*self.inner.borrow_mut() = value
	}

	pub fn replace(&self, value: T) -> T {
		std::mem::replace(&mut self.inner.borrow_mut(), value)
	}

	pub fn setter(&self) -> impl Fn(T) {
		let inner = self.inner.clone();
		move |t| {
			*inner.borrow_mut() = t;
		}
	}

	pub fn current(&self) -> std::cell::Ref<'_, T> {
		self.inner.borrow()
	}
}

impl<T> Mutable<Option<T>> {
	pub fn unwrap(&self) -> std::cell::Ref<'_, T> {
		std::cell::Ref::map(self.inner.borrow(), |r| r.as_ref().unwrap())
	}
}

pub struct Refr<B: Backend, F: Fn(Option<B::Node>)> {
	func: F,
	_b: PhantomData<B>,
}

impl<B: Backend, F: Fn(Option<B::Node>)> Markup<B> for Refr<B, F> {
	fn render(&mut self, tree: &crate::tree::Tree<B>) {
		let tree = tree.closest_node();
		(self.func)(Some(tree));
	}
	fn diff(&mut self, _prev: &mut Self, _tree: &crate::tree::Tree<B>) {}
	fn drop(&mut self, _tree: &crate::tree::Tree<B>, _should_unmount: bool) {
		(self.func)(None);
	}
}

pub fn reference<B: Backend>(func: impl Fn(Option<B::Node>)) -> impl Markup<B> {
	Refr {
		func,
		_b: PhantomData,
	}
}
