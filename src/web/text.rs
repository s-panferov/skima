use std::borrow::Cow;

use crate::tree::Tree;
use crate::web::Markup;
use crate::HtmlBackend;

pub macro impl_string($ty:ty $(, $a:lifetime )?) {
	impl<$( $a, )? B: HtmlBackend> Markup<B> for $ty {
		fn has_own_node() -> bool {
			true
		}

		fn dynamic() -> bool {
			true
		}

		fn render(&self, tree: &Tree<B>) {
			tracing::debug!("Rendering text {}", self);

			let text = tree.backend.create_text(self.as_ref());
			let prev = tree.set_node(B::text_to_node(text));
			tree.attach(prev);
		}

		fn diff(&self, prev: &Self, tree: &Tree<B>) {
			tracing::debug!("Diffing text {}", self);

			if prev != self {
				tree.backend
					.set_text(&B::node_to_text(tree.node().clone()).unwrap(), &self)
			}
		}

		fn drop(&self, tree: &Tree<B>, should_unmount: bool) {
			if !should_unmount {
				return;
			}

			B::remove(&tree.node());
		}
	}
}

impl_string!(String);
impl_string!(bumpalo::collections::String<'a>, 'a);
impl_string!(&'static str);
impl_string!(Cow<'static, str>);
