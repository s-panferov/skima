#![feature(type_alias_impl_trait)]

use bumpalo::Bump;
pub use skima::web::prelude::*;
use skima::web::ssr::StaticHtml;
use skima::web::HtmlBackend;
pub use skima_rsx::html;

fn _button<B: HtmlBackend>() -> impl Markup<B> {
	return html!(<button>"Buttom"</button>);
}

fn main() {
	let markup = html!(<div class="10">
		"Console log"
	</div>);

	let bump = Bump::new();
	let backend = StaticHtml::new(&bump);

	println!("{}", skima::web::ssr::render(backend, markup));
}
