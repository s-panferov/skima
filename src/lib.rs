#![feature(associated_type_defaults)]
#![feature(type_alias_impl_trait)]
#![feature(arbitrary_self_types)]
#![feature(core_intrinsics)]
#![feature(decl_macro)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(tuple_trait)]
#![feature(coerce_unsized, unsize)]
#![feature(provide_any)]

use std::marker::PhantomData;

use downcast_rs::Downcast;
use tree::Tree;

pub mod macros;

pub mod action;
pub mod anydata;
pub mod combine;
pub mod component;
pub mod dynamic;
pub mod effect;
pub mod ext;
mod iter;
pub mod list;
pub mod reference;
mod tap;
pub mod tree;
mod tuple;
pub mod web;

mod docs;

pub use skima_rsx::html;
pub use tap::tap;

pub use crate::dynamic::dynamic;

pub trait Markup<B: Backend = web::WebSys> {
	fn has_own_node() -> bool {
		true
	}

	fn dynamic() -> bool {
		true
	}

	fn render(&mut self, tree: &Tree<B>);
	fn diff(&mut self, prev: &mut Self, tree: &Tree<B>);
	fn drop(&mut self, tree: &Tree<B>, should_unmount: bool);
}

pub trait AnyMarkup<B: Backend = web::WebSys>: Downcast {
	fn debug(&self) -> &'static str;
	fn render(&mut self, tree: &Tree<B>);
	fn diff(&mut self, prev: &mut dyn AnyMarkup<B>, tree: &Tree<B>);
	fn drop(&mut self, tree: &Tree<B>, should_unmount: bool);
}

impl<B, T> AnyMarkup<B> for T
where
	T: Markup<B> + 'static,
	B: Backend + 'static,
{
	fn debug(&self) -> &'static str {
		if let Ok(b) = castaway::cast!(self, &Box<dyn AnyMarkup>) {
			return (**b).debug();
		}
		std::any::type_name::<T>()
	}

	fn render(&mut self, tree: &Tree<B>) {
		Markup::render(self, tree)
	}

	fn diff(&mut self, prev: &mut dyn AnyMarkup<B>, tree: &Tree<B>) {
		Markup::diff(
			self,
			prev.downcast_mut::<T>()
				.unwrap_or_else(|| panic!("{}", std::any::type_name::<T>())),
			tree,
		)
	}

	fn drop(&mut self, tree: &Tree<B>, should_unmount: bool) {
		Markup::drop(self, tree, should_unmount)
	}
}

downcast_rs::impl_downcast!(AnyMarkup<B> where B: Backend);

pub struct Context<M: Markup<B>, B: Backend> {
	markup: M,
	_b: PhantomData<B>,
}

#[inline]
pub fn render_subtree<M: Markup<B>, B: Backend>(markup: &mut M, parent: &Tree<B>) {
	if M::has_own_node() {
		let subtree = Tree::new(parent);
		markup.render(&subtree)
	} else {
		markup.render(parent)
	}
}

#[inline]
pub fn subtree<M: Markup<B>, B: Backend>(parent: &Tree<B>) -> Tree<B> {
	if M::has_own_node() {
		parent.children.borrow().first().unwrap().0.clone()
	} else {
		parent.clone()
	}
}

pub trait Backend: std::fmt::Debug + Clone {
	type Element: std::fmt::Debug + Clone;
	type Text: std::fmt::Debug + Clone;
	type Node: std::fmt::Debug + Clone;

	type Event;
	type Cursor;

	fn replace(node: &Self::Node, prev: &Self::Node);
	fn insert(cursor: Self::Cursor, node: &Self::Node);
	fn remove(node: &Self::Node);

	fn cursor_beginning_of(node: &Self::Element) -> Self::Cursor;
	fn cursor_after(node: &Self::Node) -> Self::Cursor;

	fn create_element(&self, tag: &'static str) -> Self::Element;
	fn create_text(&self, data: &str) -> Self::Text;

	fn set_text(&self, text: &Self::Text, data: &str);

	fn text_to_node(text: Self::Text) -> Self::Node;
	fn element_to_node(element: Self::Element) -> Self::Node;

	fn node_to_element(node: Self::Node) -> Option<Self::Element>;
	fn node_to_text(node: Self::Node) -> Option<Self::Text>;

	fn print_node(node: &Self::Node);
}

impl<B: Backend> Markup<B> for () {
	fn render(&mut self, _tree: &Tree<B>) {}
	fn diff(&mut self, _prev: &mut Self, _tree: &Tree<B>) {}
	fn drop(&mut self, _ree: &Tree<B>, _should_unmount: bool) {}
}

impl<M: Markup<B>, B: Backend> Context<M, B> {
	pub fn use_state<T>(&self, _initial: T) -> (T, Box<dyn Fn(T)>) {
		todo!()
	}
}

impl<M, B> Markup<B> for Option<M>
where
	M: Markup<B>,
	B: Backend,
{
	fn has_own_node() -> bool {
		M::has_own_node()
	}

	fn dynamic() -> bool {
		true
	}

	fn render(&mut self, tree: &Tree<B>) {
		if let Some(markup) = self.as_mut() {
			markup.render(tree)
		}
	}

	fn diff(&mut self, prev: &mut Self, tree: &Tree<B>) {
		match (self, prev) {
			(Some(next), None) => next.render(tree),
			(Some(next), Some(prev)) => {
				if M::dynamic() {
					next.diff(prev, tree)
				}
			}
			(None, Some(prev)) => prev.drop(tree, true),
			(None, None) => {}
		}
	}

	fn drop(&mut self, tree: &Tree<B>, should_unmount: bool) {
		match self {
			Some(markup) => markup.drop(tree, should_unmount),
			None => {}
		}
	}
}

impl<BACKEND, T> Markup<BACKEND> for Box<T>
where
	T: Markup<BACKEND>,
	BACKEND: Backend,
{
	fn has_own_node() -> bool {
		T::has_own_node()
	}

	fn dynamic() -> bool {
		T::dynamic()
	}

	fn diff(&mut self, prev: &mut Self, tree: &Tree<BACKEND>) {
		(**self).diff(prev, tree)
	}

	fn render(&mut self, tree: &Tree<BACKEND>) {
		(**self).render(tree)
	}

	fn drop(&mut self, tree: &Tree<BACKEND>, should_unmount: bool) {
		(**self).drop(tree, should_unmount)
	}
}

impl<BACKEND> Markup<BACKEND> for Box<dyn AnyMarkup<BACKEND>>
where
	BACKEND: Backend + 'static,
{
	fn has_own_node() -> bool {
		true
	}

	fn dynamic() -> bool {
		true
	}

	fn diff(&mut self, prev: &mut Self, tree: &Tree<BACKEND>) {
		(**self).diff(prev, tree)
	}

	fn render(&mut self, tree: &Tree<BACKEND>) {
		(**self).render(tree)
	}

	fn drop(&mut self, tree: &Tree<BACKEND>, should_unmount: bool) {
		(**self).drop(tree, should_unmount)
	}
}

// impl<'a, T, BACKEND> Markup<BACKEND> for &'a T
// where
// 	T: Markup<BACKEND>,
// 	BACKEND: Backend,
// {
// 	fn has_own_node() -> bool {
// 		T::has_own_node()
// 	}

// 	fn dynamic() -> bool {
// 		T::dynamic()
// 	}

// 	fn diff(&mut self, prev: &mut Self, tree: &Tree<BACKEND>) {
// 		(**self).diff(prev, tree)
// 	}

// 	fn render(&mut self, tree: &Tree<BACKEND>) {
// 		(**self).render(tree)
// 	}

// 	fn drop(&mut self, tree: &Tree<BACKEND>, should_unmount: bool) {
// 		(**self).drop(tree, should_unmount)
// 	}
// }

impl<'a, T, BACKEND> Markup<BACKEND> for &'a mut T
where
	T: Markup<BACKEND>,
	BACKEND: Backend,
{
	fn has_own_node() -> bool {
		T::has_own_node()
	}

	fn dynamic() -> bool {
		T::dynamic()
	}

	fn diff(&mut self, prev: &mut Self, tree: &Tree<BACKEND>) {
		(**self).diff(prev, tree)
	}

	fn render(&mut self, tree: &Tree<BACKEND>) {
		(**self).render(tree)
	}

	fn drop(&mut self, tree: &Tree<BACKEND>, should_unmount: bool) {
		(**self).drop(tree, should_unmount)
	}
}

// Markup for Rc was disabled. See note `notes/2023-05-07-typeid.md`
//
// impl<BACKEND, T> Markup<BACKEND> for Rc<T>
// where
// 	T: Markup<BACKEND>,
// 	BACKEND: Backend,
// {
// 	fn has_own_node() -> bool {
// 		T::has_own_node()
// 	}

// 	fn dynamic() -> bool {
// 		T::dynamic()
// 	}

// 	fn diff(&mut self, prev: &mut Self, tree: &Tree<BACKEND>) {
// 		(**self).diff(prev, tree)
// 	}

// 	fn render(&mut self, tree: &Tree<BACKEND>) {
// 		(**self).render(tree)
// 	}

// 	fn drop(&mut self, tree: &Tree<BACKEND>, should_unmount: bool) {
// 		(**self).drop(tree, should_unmount)
// 	}
// }

// impl<BACKEND> Markup<BACKEND> for Rc<dyn AnyMarkup<BACKEND>>
// where
// 	BACKEND: Backend + 'static,
// {
// 	fn has_own_node() -> bool {
// 		true
// 	}

// 	fn dynamic() -> bool {
// 		true
// 	}

// 	fn diff(&mut self, prev: &mut Self, tree: &Tree<BACKEND>) {
// 		(**self).diff(prev, tree)
// 	}

// 	fn render(&mut self, tree: &Tree<BACKEND>) {
// 		(**self).render(tree)
// 	}

// 	fn drop(&mut self, tree: &Tree<BACKEND>, should_unmount: bool) {
// 		(**self).drop(tree, should_unmount)
// 	}
// }
