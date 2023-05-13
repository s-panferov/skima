use std::marker::PhantomData;

use bumpalo::Bump;

use super::reactive::Extension;
use crate::tree::Tree;
use crate::web::reactive::{ReactiveContext, WithArena};
use crate::web::{Backend, Markup};

pub struct BumpWrapper<M, B>
where
	M: Markup<B>,
	B: Backend,
{
	bump: Bump,
	markup: M,
	_b: PhantomData<B>,
}

impl<M, B> Markup<B> for BumpWrapper<M, B>
where
	M: Markup<B>,
	B: Backend,
{
	fn render(&mut self, _tree: &Tree<B>) {
		todo!()
	}

	fn diff(&mut self, _prev: &mut Self, _tree: &Tree<B>) {
		todo!()
	}

	fn drop(&mut self, _tree: &Tree<B>, _should_unmount: bool) {
		todo!()
	}
}

pub trait MarkupWithLifetime<'a, B: Backend>: Markup<B> {}

impl<'a, B: Backend, T> MarkupWithLifetime<'a, B> for T where T: Markup<B> {}

// unsafe fn extend_lifetime<'b, B: Backend>(
// 	r: MarkupWithLifetime<'b, B>,
// ) -> MarkupWithLifetime<'static, B> {
// 	std::mem::transmute::<R<'b>, R<'static>>(r)
// }

pub fn bump<'a, B: Backend, F: FnOnce(&'a Bump) -> M, M: Markup<B> + 'a>(
	func: F,
) -> BumpWrapper<M, B> {
	let bump = bumpalo::Bump::new();
	let markup = (func)(unsafe { std::mem::transmute(&bump) });

	BumpWrapper {
		markup,
		bump,
		_b: PhantomData,
	}
}

pub struct ContextBumpWrapper<M, B>
where
	M: Markup<B>,
	B: Backend,
{
	markup: M,
	_b: PhantomData<B>,
}

impl<M, B> Markup<B> for ContextBumpWrapper<M, B>
where
	M: Markup<B>,
	B: Backend,
{
	fn render(&mut self, _tree: &Tree<B>) {
		todo!()
	}

	fn diff(&mut self, _prev: &mut Self, _tree: &Tree<B>) {
		todo!()
	}

	fn drop(&mut self, _tree: &Tree<B>, _should_unmount: bool) {
		todo!()
	}
}

pub fn bump_in<'a, 'b, B: Backend, F: FnOnce(&'a Bump) -> M, M: Markup<B> + 'a, E>(
	cx: &'b ReactiveContext<B, E>,
	func: F,
) -> ContextBumpWrapper<M, B>
where
	ReactiveContext<B, E>: Extension<WithArena>,
{
	let with_arena = cx.try_extension().unwrap();
	let markup = (func)(unsafe { std::mem::transmute(&with_arena.arena) });

	ContextBumpWrapper {
		markup,
		_b: PhantomData,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::web::html::div;
	use crate::web::WebSys;

	#[test]
	fn test() {
		let _m =
			bump(|b| div::<_, WebSys>(bumpalo::format!(in b, "String {}", 10).into_bump_str()));
	}
}
