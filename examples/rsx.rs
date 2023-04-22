#![feature(type_alias_impl_trait)]

pub use skima::web::prelude::*;
pub use skima_rsx::html;

fn main() {
	html!(<div class="10">
		"Console log"
	</div>);
}
