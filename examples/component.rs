#![feature(type_alias_impl_trait)]

use skima::component::Component;
use skima::web::prelude::*;
use web_sys::Event;

pub struct App {}

pub struct AppState {
	on_click: Callback<dyn Fn(Event)>,
}

impl Component for App {
	type State = AppState;
	type Markup = impl Markup;

	fn create(&self) -> AppState {
		AppState {
			on_click: Callback::new(|ev| {}).to_dyn(),
		}
	}

	fn render(&self, state: &mut Self::State) -> Self::Markup {
		let on_click = state.on_click.clone();
		reactive(move |cx| div(("String", on("click", on_click.clone()))))
	}
}

pub fn test() -> impl Markup {
	div(App {}.markup())
}

pub fn main() {}
