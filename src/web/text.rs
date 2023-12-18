use std::borrow::Cow;

use crate::tree::Tree;
use crate::web::Markup;

pub macro impl_string($d:expr, $ty:ty $(, $a:lifetime )?) {
	impl<$( $a, )? B: $crate::web::HtmlBackend> Markup<B> for $ty {
		fn has_own_node() -> bool {
			true
		}

		fn dynamic() -> bool {
			$d
		}

		fn render(&mut self, tree: &Tree<B>) {
			#[cfg(debug_assertions)]
			tree.name.replace(std::borrow::Cow::Borrowed("[text]"));

			let text = tree.backend.create_text(self.as_ref());
			let prev = tree.set_node(B::text_to_node(text));
			tree.attach(prev);
		}

		fn diff(&mut self, prev: &mut Self, tree: &Tree<B>) {

			if prev != self {
				tree.backend
					.set_text(&B::node_to_text(tree.node().clone()).unwrap(), &self)
			}
		}

		fn drop(&mut self, tree: &Tree<B>, should_unmount: bool) {
			if !should_unmount {
				return;
			}

			tree.unmount()
		}
	}
}

impl_string!(true, String);
impl_string!(true, bumpalo::collections::String<'a>, 'a);
impl_string!(false, &'a str, 'a);
impl_string!(true, Cow<'a, str>, 'a);
