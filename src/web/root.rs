use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::tree::Tree;
use crate::web::{Markup, WebSys};
use crate::{console_log, render_subtree, subtree};

pub struct Root<M>
where
	M: Markup<WebSys>,
{
	pub(crate) tree: Tree<WebSys>,
	markup: M,
	is_ephemeral: bool,
}

impl<M> Root<M>
where
	M: Markup<WebSys>,
{
	pub fn render(mut markup: M, container: HtmlElement) -> Root<M> {
		let tree = Tree::root(container.unchecked_into(), WebSys {});
		console_log!("Render root subtree");

		render_subtree(&mut markup, &tree);
		Root {
			markup,
			tree,
			is_ephemeral: false,
		}
	}

	pub fn ephemeral(mut markup: M) -> Root<M> {
		let tree = Tree::ephemeral_root(WebSys {});
		console_log!("Render root subtree");

		if !M::has_own_node() {
			panic!("Ephemeral Root requires markup to have an own tree node");
		}

		markup.render(&tree);

		Root {
			markup,
			tree,
			is_ephemeral: true,
		}
	}

	pub fn portal(mut markup: M, container: HtmlElement, parent: &Tree<WebSys>) -> Root<M> {
		let tree = Tree::new(parent);
		tree.set_node(container.unchecked_into());

		markup.render(&tree);

		Root {
			markup,
			tree,
			is_ephemeral: true,
		}
	}

	pub fn element(&self) -> HtmlElement {
		self.tree.node().clone().unchecked_into()
	}

	pub fn update(&mut self, mut next: M) {
		if M::dynamic() {
			let tree = if self.is_ephemeral {
				self.tree.clone()
			} else {
				subtree::<M, _>(&self.tree)
			};

			next.diff(&mut self.markup, &tree);
		}

		self.markup = next;
	}
}

impl<M> Drop for Root<M>
where
	M: Markup<WebSys>,
{
	fn drop(&mut self) {
		let tree = if self.is_ephemeral {
			self.tree.clone()
		} else {
			subtree::<M, _>(&self.tree)
		};

		self.markup.drop(&tree, false);
		self.tree
			.node()
			.unchecked_ref::<HtmlElement>()
			.set_inner_html("");
	}
}
