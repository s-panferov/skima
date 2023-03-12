use crate::tree::Tree;
use crate::{render_subtree, Backend, Markup};

impl<BACKEND, A> Markup<BACKEND> for (A,)
where
	BACKEND: Backend,
	A: Markup<BACKEND>,
{
	fn has_own_node() -> bool {
		A::has_own_node()
	}

	fn dynamic() -> bool {
		A::dynamic()
	}

	fn render(&self, tree: &Tree<BACKEND>) {
		self.0.render(tree);
	}

	fn diff(&self, prev: &Self, tree: &Tree<BACKEND>) {
		self.0.diff(&prev.0, tree);
	}

	fn drop(&self, tree: &Tree<BACKEND>, should_unmount: bool) {
		self.0.drop(tree, should_unmount);
	}
}

#[inline]
fn advance<'a, B: Backend>(
	root: &'a Tree<B>,
	cursor: &'a mut Option<Tree<B>>,
	has_node: bool,
) -> &'a Tree<B> {
	if has_node {
		*cursor = Some(
			cursor
				.as_ref()
				.map(|c| c.next())
				.unwrap_or_else(|| root.first_child()),
		);

		cursor.as_ref().unwrap()
	} else {
		root
	}
}

impl<BACKEND, A, B> Markup<BACKEND> for (A, B)
where
	BACKEND: Backend,
	A: Markup<BACKEND>,
	B: Markup<BACKEND>,
{
	fn has_own_node() -> bool {
		A::has_own_node() || B::has_own_node()
	}

	fn dynamic() -> bool {
		A::dynamic() || B::dynamic()
	}

	fn render(&self, tree: &Tree<BACKEND>) {
		render_subtree(&self.0, tree);
		render_subtree(&self.1, tree);
	}

	fn diff(&self, prev: &Self, tree: &Tree<BACKEND>) {
		let mut cursor: Option<Tree<BACKEND>> = None;

		{
			let tree = advance(tree, &mut cursor, A::has_own_node());
			if A::dynamic() {
				self.0.diff(&prev.0, tree)
			}
		}

		if B::dynamic() {
			self.1
				.diff(&prev.1, advance(tree, &mut cursor, B::has_own_node()));
		}
	}

	fn drop(&self, tree: &Tree<BACKEND>, should_unmount: bool) {
		{
			let mut cursor: Option<Tree<BACKEND>> = None;

			self.0.drop(
				advance(tree, &mut cursor, A::has_own_node()),
				should_unmount,
			);

			self.1.drop(
				advance(tree, &mut cursor, B::has_own_node()),
				should_unmount,
			);
		}

		if Self::has_own_node() {
			tree.clear()
		}
	}
}

impl<BACKEND, A, B, C> Markup<BACKEND> for (A, B, C)
where
	BACKEND: Backend,
	A: Markup<BACKEND>,
	B: Markup<BACKEND>,
	C: Markup<BACKEND>,
{
	fn has_own_node() -> bool {
		A::has_own_node() || B::has_own_node() || C::has_own_node()
	}

	fn dynamic() -> bool {
		A::dynamic() || B::dynamic() || C::dynamic()
	}

	fn render(&self, tree: &Tree<BACKEND>) {
		render_subtree(&self.0, tree);
		render_subtree(&self.1, tree);
		render_subtree(&self.2, tree);
	}

	fn diff(&self, prev: &Self, tree: &Tree<BACKEND>) {
		let mut cursor: Option<Tree<BACKEND>> = None;

		{
			let tree = advance(tree, &mut cursor, A::has_own_node());
			if A::dynamic() {
				self.0.diff(&prev.0, tree);
			}
		}

		if !B::dynamic() && !C::dynamic() {
			return;
		}

		{
			let tree = advance(tree, &mut cursor, B::has_own_node());
			if B::dynamic() {
				self.1.diff(&prev.1, tree);
			}
		}

		if !C::dynamic() {
			return;
		}

		self.2
			.diff(&prev.2, advance(tree, &mut cursor, C::has_own_node()));
	}

	fn drop(&self, tree: &Tree<BACKEND>, should_unmount: bool) {
		{
			let mut cursor: Option<Tree<BACKEND>> = None;

			self.0.drop(
				advance(tree, &mut cursor, A::has_own_node()),
				should_unmount,
			);

			self.1.drop(
				advance(tree, &mut cursor, B::has_own_node()),
				should_unmount,
			);

			self.2.drop(
				advance(tree, &mut cursor, C::has_own_node()),
				should_unmount,
			);
		}

		if Self::has_own_node() {
			tree.clear()
		}
	}
}

impl<BACKEND, A, B, C, D> Markup<BACKEND> for (A, B, C, D)
where
	BACKEND: Backend,
	A: Markup<BACKEND>,
	B: Markup<BACKEND>,
	C: Markup<BACKEND>,
	D: Markup<BACKEND>,
{
	fn has_own_node() -> bool {
		A::has_own_node() || B::has_own_node() || C::has_own_node() || D::has_own_node()
	}

	fn dynamic() -> bool {
		A::dynamic() || B::dynamic() || C::dynamic() || D::dynamic()
	}

	fn render(&self, tree: &Tree<BACKEND>) {
		render_subtree(&self.0, tree);
		render_subtree(&self.1, tree);
		render_subtree(&self.2, tree);
		render_subtree(&self.3, tree);
	}

	fn diff(&self, prev: &Self, tree: &Tree<BACKEND>) {
		let mut cursor: Option<Tree<BACKEND>> = None;

		{
			let tree = advance(tree, &mut cursor, A::has_own_node());
			if A::dynamic() {
				self.0.diff(&prev.0, tree);
			}
		}

		if !B::dynamic() && !C::dynamic() && !D::dynamic() {
			return;
		}

		{
			let tree = advance(tree, &mut cursor, B::has_own_node());
			if B::dynamic() {
				self.1.diff(&prev.1, tree);
			}
		}

		if !C::dynamic() && !D::dynamic() {
			return;
		}

		{
			let tree = advance(tree, &mut cursor, C::has_own_node());
			if C::dynamic() {
				self.2.diff(&prev.2, tree);
			}
		}

		if !D::dynamic() {
			return;
		}

		{
			let tree = advance(tree, &mut cursor, D::has_own_node());
			self.3.diff(&prev.3, tree);
		}
	}

	fn drop(&self, tree: &Tree<BACKEND>, should_unmount: bool) {
		{
			let mut cursor: Option<Tree<BACKEND>> = None;

			self.0.drop(
				advance(tree, &mut cursor, A::has_own_node()),
				should_unmount,
			);

			self.1.drop(
				advance(tree, &mut cursor, B::has_own_node()),
				should_unmount,
			);

			self.2.drop(
				advance(tree, &mut cursor, C::has_own_node()),
				should_unmount,
			);

			self.3.drop(
				advance(tree, &mut cursor, D::has_own_node()),
				should_unmount,
			);
		}

		if Self::has_own_node() {
			tree.clear()
		}
	}
}

impl<BACKEND, A, B, C, D, E> Markup<BACKEND> for (A, B, C, D, E)
where
	BACKEND: Backend,
	A: Markup<BACKEND>,
	B: Markup<BACKEND>,
	C: Markup<BACKEND>,
	D: Markup<BACKEND>,
	E: Markup<BACKEND>,
{
	fn has_own_node() -> bool {
		A::has_own_node()
			|| B::has_own_node()
			|| C::has_own_node()
			|| D::has_own_node()
			|| E::has_own_node()
	}

	fn dynamic() -> bool {
		A::dynamic() || B::dynamic() || C::dynamic() || D::dynamic() || E::dynamic()
	}

	fn render(&self, tree: &Tree<BACKEND>) {
		render_subtree(&self.0, tree);
		render_subtree(&self.1, tree);
		render_subtree(&self.2, tree);
		render_subtree(&self.3, tree);
		render_subtree(&self.4, tree);
	}

	fn diff(&self, prev: &Self, tree: &Tree<BACKEND>) {
		let mut cursor: Option<Tree<BACKEND>> = None;

		{
			let tree = advance(tree, &mut cursor, A::has_own_node());
			if A::dynamic() {
				self.0.diff(&prev.0, tree);
			}
		}

		if !B::dynamic() && !C::dynamic() && !D::dynamic() && !E::dynamic() {
			return;
		}

		{
			let tree = advance(tree, &mut cursor, B::has_own_node());
			if B::dynamic() {
				self.1.diff(&prev.1, tree);
			}
		}

		if !C::dynamic() && !D::dynamic() && !E::dynamic() {
			return;
		}

		{
			let tree = advance(tree, &mut cursor, C::has_own_node());
			if C::dynamic() {
				self.2.diff(&prev.2, tree);
			}
		}

		if !D::dynamic() || !E::dynamic() {
			return;
		}

		{
			let tree = advance(tree, &mut cursor, D::has_own_node());
			if D::dynamic() {
				self.3.diff(&prev.3, tree);
			}
		}

		if !E::dynamic() {
			return;
		}

		{
			let tree = advance(tree, &mut cursor, E::has_own_node());
			self.4.diff(&prev.4, tree);
		}
	}

	fn drop(&self, tree: &Tree<BACKEND>, should_unmount: bool) {
		{
			let mut cursor: Option<Tree<BACKEND>> = None;

			self.0.drop(
				advance(tree, &mut cursor, A::has_own_node()),
				should_unmount,
			);

			self.1.drop(
				advance(tree, &mut cursor, B::has_own_node()),
				should_unmount,
			);

			self.2.drop(
				advance(tree, &mut cursor, C::has_own_node()),
				should_unmount,
			);

			self.3.drop(
				advance(tree, &mut cursor, D::has_own_node()),
				should_unmount,
			);

			self.4.drop(
				advance(tree, &mut cursor, E::has_own_node()),
				should_unmount,
			);
		}

		if Self::has_own_node() {
			tree.clear()
		}
	}
}
