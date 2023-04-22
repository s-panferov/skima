use skima::web::bump::bump;
use skima::web::html::*;
use skima::Markup;
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn bump_cx() -> impl Markup {
	bump(|cx| {
		let str = bumpalo::format!(in cx, "String {}", 10).into_bump_str();
		div(str)
	})
}

#[wasm_bindgen_test]
fn test_bump() {
	let document = web_sys::window().unwrap().document().unwrap();
	let _container = document.create_element("div").unwrap();

	// let root = Root::render(bump_cx());
}
