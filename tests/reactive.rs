use observe::{batch, Var};
use skima::web::event::on;
use skima::web::html::div;
use skima::web::reactive::reactive;
use skima::web::root::Root;
use skima::Markup;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn component() -> impl Markup {
	reactive(|cx| {
		cx.with(Var::new(true));

		let var = cx.get::<Var<bool>>();
		let text = if var.get(cx) { "True" } else { "False" };

		div((
			text,
			on(
				"click",
				cx.wrap_1(|cx, _e| {
					batch(|| cx.get::<Var<bool>>().set(false));
				}),
			),
		))
	})
}

#[wasm_bindgen_test]
pub fn test_reactive() {
	let document = web_sys::window().unwrap().document().unwrap();
	let container = document.create_element("div").unwrap();

	let root = Root::render(component(), container.unchecked_into());
	std::mem::forget(root);
}
