use std::marker::PhantomData;

use super::HtmlBackend;
use crate::tree::Tree;
use crate::web::{Backend, Markup};
use crate::{render_subtree, subtree};

#[derive(Clone)]
pub struct Tag<M: Markup<B>, B: Backend, const N: usize> {
	tag: &'static str,
	markup: M,
	_b: PhantomData<B>,
}

impl<M, B: HtmlBackend, const N: usize> Markup<B> for Tag<M, B, N>
where
	M: Markup<B>,
{
	fn has_own_node() -> bool {
		true
	}

	fn dynamic() -> bool {
		M::dynamic()
	}

	fn render(&self, tree: &Tree<B>) {
		tracing::debug!("Rendering tag {}", self.tag);

		let node = B::element_to_node(tree.backend.create_element(self.tag));
		let prev = tree.set_node(node);
		render_subtree(&self.markup, tree);
		tree.attach(prev);
	}

	fn diff(&self, prev: &Self, tree: &Tree<B>) {
		if prev.tag != self.tag {
			// re-render
			let element = B::element_to_node(tree.backend.create_element(self.tag));

			let prev = tree.set_node(element.into());
			tree.clear();

			render_subtree(&self.markup, tree);
			tree.attach(prev)
		} else if M::dynamic() {
			self.markup.diff(&prev.markup, &subtree::<M, _>(tree));
		}
	}

	fn drop(&self, tree: &Tree<B>, should_unmount: bool) {
		tracing::debug!("Undo tag");
		if M::has_own_node() {
			self.markup.drop(&tree.first_child(), false);
		} else {
			self.markup.drop(tree, false);
		}

		tree.clear();

		if should_unmount {
			B::remove(&tree.node());
		}
	}
}

pub macro define_tag($name:ident, $ty:literal) {
	pub fn $name<M: Markup<B>, B: Backend>(markup: M) -> Tag<M, B, 1>
	where
		Tag<M, B, 1>: Markup<B>,
	{
		Tag {
			tag: stringify!($name),
			markup,
			_b: PhantomData,
		}
	}
}

pub mod html {
	use super::define_tag;

	define_tag!(div, 1);
	define_tag!(span, 2);
	define_tag!(a, 3);
	define_tag!(input, 4);
	define_tag!(br, 5);
	define_tag!(img, 6);
	define_tag!(button, 7);
	define_tag!(label, 8);
	define_tag!(textarea, 9);
	define_tag!(h1, 10);
	define_tag!(h2, 11);
	define_tag!(h3, 12);
	define_tag!(header, 13);
	define_tag!(section, 14);
	define_tag!(ul, 15);
	define_tag!(li, 16);
	define_tag!(video, 17);
	define_tag!(source, 18);
	define_tag!(html, 19);
	define_tag!(head, 20);
	define_tag!(script, 20);
	define_tag!(link, 21);
	define_tag!(body, 22);
}
