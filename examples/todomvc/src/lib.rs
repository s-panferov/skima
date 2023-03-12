use std::rc::Rc;

use action::TodoCreate;
use skima::action::Action;
use skima::web::dispatch::{capture, ActionResult};
use skima::web::root::Root;
use skima::web::Callback;
use wasm_bindgen::prelude::*;

mod action;
mod app;

#[wasm_bindgen(start)]
fn main() -> Result<(), JsError> {
	let el = web_sys::window()
		.unwrap()
		.document()
		.unwrap()
		.body()
		.unwrap();

	let on_action = Callback::new(|action: Box<dyn Action>| {
		if let Some(action) = action.downcast_ref::<TodoCreate>() {};
		ActionResult::Stop
	});

	let root = Root::render(capture(app::app(), on_action.to_dyn()), el);

	// Otherwise our markup will be dropped
	std::mem::forget(root);
	Ok(())
}
