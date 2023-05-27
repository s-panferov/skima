use crate::tree::Tree;
use crate::{Backend, Markup};

pub enum Either<T1, T2> {
	A(T1),
	B(T2),
}

impl<T1, T2> Either<T1, T2> {
	pub fn a_tree<B>(tree: &Tree<B>) -> &Tree<B>
	where
		Self: Markup<B>,
		T1: Markup<B>,
		T2: Markup<B>,
		B: Backend,
	{
		if Self::has_own_node() {
			if T1::has_own_node() {
				tree
			} else {
				tree.parent.as_ref().unwrap()
			}
		} else {
			tree
		}
	}

	pub fn b_tree<B>(tree: &Tree<B>) -> &Tree<B>
	where
		Self: Markup<B>,
		T1: Markup<B>,
		T2: Markup<B>,
		B: Backend,
	{
		if Self::has_own_node() {
			if T2::has_own_node() {
				tree
			} else {
				tree.parent.as_ref().unwrap()
			}
		} else {
			tree
		}
	}
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

	fn has_own_node() -> bool {
		T1::has_own_node() || T2::has_own_node()
	}

	fn render(&mut self, tree: &Tree<B>) {
		// We need to be careful with the `tree` here, because
		// we'll receive a "child" tree

		match self {
			Either::A(a) => {
				a.render(Self::a_tree(tree));
			}
			Either::B(b) => {
				b.render(Self::b_tree(tree));
			}
		}
	}

	fn diff(&mut self, prev: &mut Self, tree: &Tree<B>) {
		match (self, prev) {
			(Either::A(next), Either::A(prev)) => {
				if T1::dynamic() {
					next.diff(prev, Self::a_tree(tree))
				}
			}
			(Either::A(next), Either::B(prev)) => {
				prev.drop(Self::b_tree(tree), true);
				next.render(Self::a_tree(tree));
			}
			(Either::B(next), Either::B(prev)) => {
				if T2::dynamic() {
					next.diff(prev, Self::b_tree(tree))
				}
			}
			(Either::B(next), Either::A(prev)) => {
				prev.drop(Self::a_tree(tree), true);
				next.render(Self::b_tree(tree));
			}
		}
	}

	fn drop(&mut self, tree: &Tree<B>, should_unmount: bool) {
		match self {
			Either::A(a) => a.drop(Self::a_tree(tree), should_unmount),
			Either::B(b) => b.drop(Self::b_tree(tree), should_unmount),
		}
	}
}
