#![feature(type_alias_impl_trait)]

pub use skima::web::prelude::*;
use skima::web::ssr::{render, StaticHtml};
use skima::web::tag::Tag;
use skima::web::HtmlBackend;

pub fn test<B: HtmlBackend>() -> impl Markup<B> {
	return div(());
}

fn main() {
	let bump = bumpalo::Bump::new();
	let html = StaticHtml::new(&bump);

	let markup = bump.alloc_with(|| {
		div(div(div((classname("test"), property("display", "block"))))) as Tag<_, StaticHtml, 1>
	});

	println!("{}", render(html, markup));
}
