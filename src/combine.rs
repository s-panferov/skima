use crate::{Backend, Markup};

pub enum Either<T1, T2> {
	A(T1),
	B(T2),
}

impl<T1, T2, B> Markup<B> for Either<T1, T2>
where
	T1: Markup<B>,
	T2: Markup<B>,
	B: Backend,
{
	fn dynamic() -> bool {
		T1::dynamic() || T2::dynamic()
	}

	fn render(&self, tree: &crate::tree::Tree<B>) {
		match self {
			Either::A(a) => a.render(tree),
			Either::B(b) => b.render(tree),
		}
	}

	fn diff(&self, prev: &Self, tree: &crate::tree::Tree<B>) {
		match (self, prev) {
			(Either::A(next), Either::A(prev)) => {
				if T1::dynamic() {
					next.diff(prev, tree)
				}
			}
			(Either::A(next), Either::B(prev)) => {
				// We do not unmount here to allow next node to replace previous
				prev.drop(tree, false);
				next.render(tree);
			}
			(Either::B(next), Either::B(prev)) => {
				if T2::dynamic() {
					next.diff(prev, tree)
				}
			}
			(Either::B(next), Either::A(prev)) => {
				prev.drop(tree, false);
				next.render(tree);
			}
		}
	}

	fn drop(&self, tree: &crate::tree::Tree<B>, should_unmount: bool) {
		match self {
			Either::A(a) => a.drop(tree, should_unmount),
			Either::B(b) => b.drop(tree, should_unmount),
		}
	}
}
