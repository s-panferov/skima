#![feature(associated_type_defaults)]
#![feature(type_alias_impl_trait)]
#![feature(arbitrary_self_types)]
#![feature(core_intrinsics)]
#![feature(decl_macro)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(tuple_trait)]

use std::marker::PhantomData;
use std::rc::Rc;

use tree::Tree;

pub mod macros;

pub mod action;
pub mod combine;
pub mod ext;
mod iter;
pub mod list;
pub mod reference;
pub mod tree;
pub mod web;

pub trait Markup<B: Backend = web::WebSys> {
	fn has_own_node() -> bool {
		return true;
	}

	fn render(&self, tree: &Tree<B>);
	fn diff(&self, prev: &Self, tree: &Tree<B>);
	fn drop(&self, tree: &Tree<B>, should_unmount: bool);
}

pub struct Context<M: Markup<B>, B: Backend> {
	markup: M,
	_b: PhantomData<B>,
}

#[inline]
pub fn render_subtree<M: Markup<B>, B: Backend>(markup: &M, parent: &Tree<B>) {
	if M::has_own_node() {
		let subtree = Tree::new(&parent);
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

pub trait Backend: std::fmt::Debug {
	type Node: std::fmt::Debug + Clone;
	type Event;
	type Cursor;

	fn replace(node: &Self::Node, prev: &Self::Node);
	fn insert(cursor: Self::Cursor, node: &Self::Node);

	fn cursor_beginning_of(node: &Self::Node) -> Self::Cursor;
	fn cursor_after(node: &Self::Node) -> Self::Cursor;
}

impl<B: Backend> Markup<B> for () {
	fn render(&self, _tree: &Tree<B>) {}
	fn diff(&self, _prev: &Self, _tree: &Tree<B>) {}
	fn drop(&self, _ree: &Tree<B>, _should_unmount: bool) {}
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

	fn render(&self, tree: &Tree<B>) {
		if let Some(markup) = self.as_ref() {
			markup.render(tree)
		}
	}

	fn diff(&self, prev: &Self, tree: &Tree<B>) {
		match (self, prev) {
			(Some(next), None) => next.render(tree),
			(Some(next), Some(prev)) => next.diff(prev, tree),
			(None, Some(prev)) => prev.drop(tree, true),
			(None, None) => {}
		}
	}

	fn drop(&self, tree: &Tree<B>, should_unmount: bool) {
		match self {
			Some(markup) => markup.drop(tree, should_unmount),
			None => {}
		}
	}
}

impl<BACKEND, A> Markup<BACKEND> for (A,)
where
	BACKEND: Backend,
	A: Markup<BACKEND>,
{
	fn has_own_node() -> bool {
		A::has_own_node()
	}

	fn render(&self, tree: &Tree<BACKEND>) {
		self.0.render(&tree);
	}

	fn diff(&self, prev: &Self, tree: &Tree<BACKEND>) {
		self.0.diff(&prev.0, tree);
	}

	fn drop(&self, tree: &Tree<BACKEND>, should_unmount: bool) {
		self.0.drop(&tree, should_unmount);
	}
}

#[inline]
fn advance<'a, B: Backend>(
	root: &'a Tree<B>,
	cursor: &'a mut Option<Tree<B>>,
	has_node: bool,
) -> &'a Tree<B> {
	if has_node {
		*cursor = Some(
			cursor
				.as_ref()
				.map(|c| c.next())
				.unwrap_or_else(|| root.first_child()),
		);

		cursor.as_ref().unwrap()
	} else {
		root
	}
}

impl<BACKEND, A, B> Markup<BACKEND> for (A, B)
where
	BACKEND: Backend,
	A: Markup<BACKEND>,
	B: Markup<BACKEND>,
{
	fn has_own_node() -> bool {
		A::has_own_node() || B::has_own_node()
	}

	fn render(&self, tree: &Tree<BACKEND>) {
		render_subtree(&self.0, &tree);
		render_subtree(&self.1, &tree);
	}

	fn diff(&self, prev: &Self, tree: &Tree<BACKEND>) {
		let mut cursor: Option<Tree<BACKEND>> = None;

		self.0
			.diff(&prev.0, advance(tree, &mut cursor, A::has_own_node()));
		self.1
			.diff(&prev.1, advance(tree, &mut cursor, B::has_own_node()));
	}

	fn drop(&self, tree: &Tree<BACKEND>, should_unmount: bool) {
		{
			let mut cursor: Option<Tree<BACKEND>> = None;

			self.0.drop(
				advance(tree, &mut cursor, A::has_own_node()),
				should_unmount,
			);

			self.1.drop(
				advance(tree, &mut cursor, B::has_own_node()),
				should_unmount,
			);
		}

		if Self::has_own_node() {
			tree.clear()
		}
	}
}

impl<BACKEND, A, B, C> Markup<BACKEND> for (A, B, C)
where
	BACKEND: Backend,
	A: Markup<BACKEND>,
	B: Markup<BACKEND>,
	C: Markup<BACKEND>,
{
	fn has_own_node() -> bool {
		A::has_own_node() || B::has_own_node() || C::has_own_node()
	}

	fn render(&self, tree: &Tree<BACKEND>) {
		render_subtree(&self.0, &tree);
		render_subtree(&self.1, &tree);
		render_subtree(&self.2, &tree);
	}

	fn diff(&self, prev: &Self, tree: &Tree<BACKEND>) {
		let mut cursor: Option<Tree<BACKEND>> = None;

		self.0
			.diff(&prev.0, advance(tree, &mut cursor, A::has_own_node()));
		self.1
			.diff(&prev.1, advance(tree, &mut cursor, B::has_own_node()));
		self.2
			.diff(&prev.2, advance(tree, &mut cursor, C::has_own_node()));
	}

	fn drop(&self, tree: &Tree<BACKEND>, should_unmount: bool) {
		{
			let mut cursor: Option<Tree<BACKEND>> = None;

			self.0.drop(
				advance(tree, &mut cursor, A::has_own_node()),
				should_unmount,
			);

			self.1.drop(
				advance(tree, &mut cursor, B::has_own_node()),
				should_unmount,
			);

			self.2.drop(
				advance(tree, &mut cursor, C::has_own_node()),
				should_unmount,
			);
		}

		if Self::has_own_node() {
			tree.clear()
		}
	}
}

impl<BACKEND, A, B, C, D> Markup<BACKEND> for (A, B, C, D)
where
	BACKEND: Backend,
	A: Markup<BACKEND>,
	B: Markup<BACKEND>,
	C: Markup<BACKEND>,
	D: Markup<BACKEND>,
{
	fn has_own_node() -> bool {
		A::has_own_node() || B::has_own_node() || C::has_own_node() || D::has_own_node()
	}

	fn render(&self, tree: &Tree<BACKEND>) {
		render_subtree(&self.0, &tree);
		render_subtree(&self.1, &tree);
		render_subtree(&self.2, &tree);
		render_subtree(&self.3, &tree);
	}

	fn diff(&self, prev: &Self, tree: &Tree<BACKEND>) {
		let mut cursor: Option<Tree<BACKEND>> = None;

		self.0
			.diff(&prev.0, advance(tree, &mut cursor, A::has_own_node()));
		self.1
			.diff(&prev.1, advance(tree, &mut cursor, B::has_own_node()));
		self.2
			.diff(&prev.2, advance(tree, &mut cursor, C::has_own_node()));
		self.3
			.diff(&prev.3, advance(tree, &mut cursor, D::has_own_node()));
	}

	fn drop(&self, tree: &Tree<BACKEND>, should_unmount: bool) {
		{
			let mut cursor: Option<Tree<BACKEND>> = None;

			self.0.drop(
				advance(tree, &mut cursor, A::has_own_node()),
				should_unmount,
			);

			self.1.drop(
				advance(tree, &mut cursor, B::has_own_node()),
				should_unmount,
			);

			self.2.drop(
				advance(tree, &mut cursor, C::has_own_node()),
				should_unmount,
			);

			self.3.drop(
				advance(tree, &mut cursor, D::has_own_node()),
				should_unmount,
			);
		}

		if Self::has_own_node() {
			tree.clear()
		}
	}
}

impl<BACKEND, A, B, C, D, E> Markup<BACKEND> for (A, B, C, D, E)
where
	BACKEND: Backend,
	A: Markup<BACKEND>,
	B: Markup<BACKEND>,
	C: Markup<BACKEND>,
	D: Markup<BACKEND>,
	E: Markup<BACKEND>,
{
	fn has_own_node() -> bool {
		A::has_own_node()
			|| B::has_own_node()
			|| C::has_own_node()
			|| D::has_own_node()
			|| E::has_own_node()
	}

	fn render(&self, tree: &Tree<BACKEND>) {
		render_subtree(&self.0, &tree);
		render_subtree(&self.1, &tree);
		render_subtree(&self.2, &tree);
		render_subtree(&self.3, &tree);
		render_subtree(&self.4, &tree);
	}

	fn diff(&self, prev: &Self, tree: &Tree<BACKEND>) {
		let mut cursor: Option<Tree<BACKEND>> = None;

		self.0
			.diff(&prev.0, advance(tree, &mut cursor, A::has_own_node()));
		self.1
			.diff(&prev.1, advance(tree, &mut cursor, B::has_own_node()));
		self.2
			.diff(&prev.2, advance(tree, &mut cursor, C::has_own_node()));
		self.3
			.diff(&prev.3, advance(tree, &mut cursor, D::has_own_node()));
		self.4
			.diff(&prev.4, advance(tree, &mut cursor, E::has_own_node()));
	}

	fn drop(&self, tree: &Tree<BACKEND>, should_unmount: bool) {
		{
			let mut cursor: Option<Tree<BACKEND>> = None;

			self.0.drop(
				advance(tree, &mut cursor, A::has_own_node()),
				should_unmount,
			);

			self.1.drop(
				advance(tree, &mut cursor, B::has_own_node()),
				should_unmount,
			);

			self.2.drop(
				advance(tree, &mut cursor, C::has_own_node()),
				should_unmount,
			);

			self.3.drop(
				advance(tree, &mut cursor, D::has_own_node()),
				should_unmount,
			);

			self.4.drop(
				advance(tree, &mut cursor, E::has_own_node()),
				should_unmount,
			);
		}

		if Self::has_own_node() {
			tree.clear()
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

	fn diff(&self, prev: &Self, tree: &Tree<BACKEND>) {
		(**self).diff(prev, tree)
	}

	fn render(&self, tree: &Tree<BACKEND>) {
		(**self).render(tree)
	}

	fn drop(&self, tree: &Tree<BACKEND>, should_unmount: bool) {
		(**self).drop(tree, should_unmount)
	}
}

impl<BACKEND, T> Markup<BACKEND> for Rc<T>
where
	T: Markup<BACKEND>,
	BACKEND: Backend,
{
	fn has_own_node() -> bool {
		T::has_own_node()
	}

	fn diff(&self, prev: &Self, tree: &Tree<BACKEND>) {
		(**self).diff(&*prev, tree)
	}

	fn render(&self, tree: &Tree<BACKEND>) {
		(**self).render(tree)
	}

	fn drop(&self, tree: &Tree<BACKEND>, should_unmount: bool) {
		(**self).drop(tree, should_unmount)
	}
}
