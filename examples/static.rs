#![feature(type_alias_impl_trait)]

use skima::r#static::StaticHtml;
use skima::tree::Tree;
pub use skima::web::prelude::*;
use skima::web::tag::Tag;
use skima::{Backend, HtmlBackend};
pub use skima_rsx::html;

pub fn test<B: HtmlBackend>() -> impl Markup<B> {
	return div(());
}

fn main() {
	let bump = bumpalo::Bump::new();
	let html = StaticHtml { bump: &bump };

	let markup = bump.alloc_with(|| {
		div(div(div((classname("test"), property("display", "block"))))) as Tag<_, StaticHtml, 1>
	});

	let tree = Tree::ephemeral_root(html);

	div(markup).render(&tree);

	let mut buffer = String::new();

	StaticHtml::node_to_element(tree.node().clone())
		.unwrap()
		.to_html(&mut buffer)
		.unwrap();

	println!("{}", buffer);
}
