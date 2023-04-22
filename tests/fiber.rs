use skima::tree::Tree;
use skima::web::WebSys;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
pub fn test_fiber() {
	let document = web_sys::window().unwrap().document().unwrap();
	let container = document.create_element("div").unwrap();
	let _tree: Tree<WebSys> = Tree::root(container.unchecked_into(), ());
}
