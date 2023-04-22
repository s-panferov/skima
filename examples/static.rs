#![feature(type_alias_impl_trait)]

use skima::r#static::StaticHtml;
use skima::tree::Tree;
pub use skima::web::prelude::*;
use skima::web::tag::Tag;
pub use skima_rsx::html;

fn main() {
	let bump = bumpalo::Bump::new();
	let html = StaticHtml { bump: &bump };

	let markup = div(div(div(()))) as Tag<_, StaticHtml, 1>;
	let tree = Tree::ephemeral_root(html);

	markup.render(&tree);

	let mut buffer = String::new();
	tree.node().to_html(&mut buffer).unwrap();

	println!("{:#?}", buffer);
}
