use std::marker::PhantomData;

use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::tree::Tree;
use crate::web::helpers::dom::DOCUMENT;
use crate::web::{Backend, Markup, WebSys};
use crate::{console_log, render_subtree, subtree};

#[derive(Clone)]
pub struct Tag<M: Markup<B>, B: Backend, const N: usize> {
	tag: &'static str,
	markup: M,
	_b: PhantomData<B>,
}

impl<M, const N: usize> Markup<WebSys> for Tag<M, WebSys, N>
where
	M: Markup<WebSys>,
{
	fn has_own_node() -> bool {
		true
	}

	fn render(&self, tree: &Tree<WebSys>) {
		tracing::info!("Rendering tag {}", self.tag);

		let element = DOCUMENT.with(|d| d.create_element(self.tag).unwrap());
		let prev = tree.set_node(element.unchecked_into());
		render_subtree(&self.markup, &tree);
		tree.attach(prev);
	}

	fn diff(&self, prev: &Self, tree: &Tree<WebSys>) {
		if prev.tag != self.tag {
			// re-render
			let element = DOCUMENT.with(|d| d.create_element(self.tag).unwrap());

			let prev = tree.set_node(element.into());
			tree.clear();

			render_subtree(&self.markup, &tree);
			tree.attach(prev)
		} else {
			self.markup.diff(&prev.markup, &subtree::<M, _>(tree));
		}
	}

	fn drop(&self, tree: &Tree<WebSys>, should_unmount: bool) {
		console_log!("Undo tag");

		if M::has_own_node() {
			self.markup.drop(&tree.first_child(), false);
		} else {
			self.markup.drop(tree, false);
		}

		tree.clear();

		if should_unmount {
			tree.node().unchecked_ref::<HtmlElement>().remove()
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
}
