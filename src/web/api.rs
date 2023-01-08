use std::borrow::Cow;

use bumpalo::Bump;

use crate::tree::Tree;
use crate::web::event::on;
use crate::web::reactive::reactive;
use crate::web::tag::html::*;
use crate::web::{attr, classname, Backend, Counter, Markup, Target, WebSys};
use crate::Context;

struct LinkProps {
	title: String,
	href: String,
}

fn test(props: LinkProps) -> impl Markup {
	a((
		classname("link"),
		attr("href", props.href),
		span(props.title),
		div(()),
		div(()),
	))
}

fn list<'a, L: Iterator<Item = I> + 'a, I, M: Markup<B>, F: Fn(I) -> M, B: Backend>(
	items: L,
	f: F,
) -> impl Markup<B> {
	todo!()
}

fn items<'a>(items: &'a [u64]) -> impl Markup + 'a {
	div(list(items.iter(), |item| {}))
}

fn test2() -> impl Markup {
	(
		test(LinkProps {
			title: "Test".into(),
			href: "test".into(),
		}),
		classname("asdfasdf"),
		attr("10", "23141234"),
	)
}

// h::div()
// 			.attr("class", "filetree-actions__icon")
// 			.attr("title", "Create document")
// 			.child(&h::span().attr("class", "typcn typcn-document-add"))
// 			.on(
// 				"click",
// 				cx.use_listener(|cx, _ev| {
// 					cx.state
// 						.props
// 						.dispatcher
// 						.dispatch(CreateDialogOperation::Open(
// 							CreateDialogMode::CreateDocument,
// 						))
// 				}),
// 			)
// 			.finish()

fn memo<F: Fn(Context<M, B>) -> M, M: Markup<B>, B: Backend>(func: F) -> impl Markup<B> {
	todo!()
}

fn state<T, F>() {}

fn create_document() -> impl Markup {
	reactive(|cx| {
		cx.with::<Counter>(Counter(0));

		let active: &Counter = cx.get();

		(div((
			classname("filetree-actions__icon"),
			attr("title", "Create document"),
			(
				span(classname("typcn tlo1aww1;;.....,,,ypcn-document-add")),
				on("click", cx.wrap(move |cx, _| cx.set(false))),
				title(None),
				(if true {
					("test", "test", Some(""))
				} else {
					("test", "test", None)
				}),
			),
		)),)
	})
}

fn title(title: Option<String>) -> impl Markup<Target> {
	reactive(move |cx| {
		div((
			if title.is_some() {
				Cow::Owned(title.clone().unwrap())
			} else {
				Cow::Borrowed("asdfasdf")
			},
			on("click", |click| {}),
		))
	})
}

fn root(name: &'static str) -> impl Markup {
	(
		div(div(("Hello ", name))),
		div(div(("Hello ", name))),
		div(div(("Hello ", name))),
	)
}

thread_local! {
	pub static BUMP: Bump = Bump::new();
}

fn root_bump<'a>(bump: &'a Bump, n: usize) -> impl Markup + 'a {
	let text = bumpalo::format!(in bump, "Hello {}", n).into_bump_str();
	(div(div(text)), span(span(text)), a(a(text)))
}

fn button<'a>(name: &'a str) -> impl Markup + 'a {
	reactive(|cx| div(()))
}
