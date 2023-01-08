/// Extends `Iterator` with an adapter that is peekable from both ends.
pub trait IteratorExt: Iterator {
	fn de_peekable(self) -> DoubleEndedPeekable<Self>
	where
		Self: Sized + DoubleEndedIterator,
	{
		DoubleEndedPeekable {
			iter: self,
			front: None,
			back: None,
		}
	}
}

impl<T: Iterator> IteratorExt for T {}

/// A double ended iterator that is peekable.
#[derive(Clone, Debug)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct DoubleEndedPeekable<I: Iterator> {
	iter: I,
	front: Option<Option<I::Item>>,
	back: Option<Option<I::Item>>,
}

impl<I: Iterator> DoubleEndedPeekable<I> {
	#[inline]
	pub fn peek(&mut self) -> Option<&I::Item> {
		if self.front.is_none() {
			self.front = Some(
				self.iter
					.next()
					.or_else(|| self.back.take().unwrap_or(None)),
			);
		}
		match self.front {
			Some(Some(ref value)) => Some(value),
			Some(None) => None,
			_ => unreachable!(),
		}
	}

	#[inline]
	pub fn peek_back(&mut self) -> Option<&I::Item>
	where
		I: DoubleEndedIterator,
	{
		if self.back.is_none() {
			self.back = Some(
				self.iter
					.next_back()
					.or_else(|| self.front.take().unwrap_or(None)),
			);
		}
		match self.back {
			Some(Some(ref value)) => Some(value),
			Some(None) => None,
			_ => unreachable!(),
		}
	}
}

impl<I: Iterator> Iterator for DoubleEndedPeekable<I> {
	type Item = I::Item;
	#[inline]
	fn next(&mut self) -> Option<I::Item> {
		self.front
			.take()
			.unwrap_or_else(|| self.iter.next())
			.or_else(|| self.back.take().unwrap_or(None))
	}

	#[inline]
	fn size_hint(&self) -> (usize, Option<usize>) {
		let peek_len = match self.front {
			Some(None) => return (0, Some(0)),
			Some(Some(_)) => 1,
			None => 0,
		} + match self.back {
			Some(None) => return (0, Some(0)),
			Some(Some(_)) => 1,
			None => 0,
		};
		let (lo, hi) = self.iter.size_hint();
		(
			lo.saturating_add(peek_len),
			hi.and_then(|x| x.checked_add(peek_len)),
		)
	}
}

impl<I: DoubleEndedIterator> DoubleEndedIterator for DoubleEndedPeekable<I> {
	#[inline]
	fn next_back(&mut self) -> Option<Self::Item> {
		self.back
			.take()
			.unwrap_or_else(|| self.iter.next_back())
			.or_else(|| self.front.take().unwrap_or(None))
	}
}

impl<I: std::iter::ExactSizeIterator> std::iter::ExactSizeIterator for DoubleEndedPeekable<I> {}
impl<I: std::iter::FusedIterator> std::iter::FusedIterator for DoubleEndedPeekable<I> {}
