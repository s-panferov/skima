use castaway::cast;

use crate::dont_panic;

struct Context<E> {
	ext: E,
}

impl<T: 'static, A: 'static> AsRef<T> for Context<(A,)> {
	fn as_ref(&self) -> &T {
		if let Ok(t) = cast!(&self.ext.0, &T) {
			return t;
		}

		dont_panic!()
	}
}

impl<T: 'static, A: 'static, B: 'static> AsRef<T> for Context<(A, B)> {
	fn as_ref(&self) -> &T {
		if let Ok(t) = cast!(&self.ext.0, &T) {
			return t;
		} else if let Ok(t) = cast!(&self.ext.1, &T) {
			return t;
		}

		dont_panic!()
	}
}

struct Arena {}

impl<E> Context<E> {
	fn arena(&self)
	where
		Self: AsRef<Arena>,
	{
		let a: &Arena = self.as_ref();
	}
}

fn test(cx: Context<(Arena,)>) {
	cx.arena()
}
