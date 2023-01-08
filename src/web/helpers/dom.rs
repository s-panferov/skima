thread_local! {
	pub static DOCUMENT: web_sys::Document = web_sys::window().unwrap().document().unwrap();
	pub static WINDOW: web_sys::Window = web_sys::window().unwrap();
}
