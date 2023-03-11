## Store closures in memo

An interesting case of working with `Memo` objects
is to store closures there.

```
reactive(|cx| {
  let c1 = cx.with_memo(|| Closure::new(|v: JsValue| {}))
  let c2 = cx.with_memo(|| Closure::new(|v: JsValue| {}))
})
```

The problem here is that we'll get the **same** memo in `c1` and `c2` becase both closure functions has the same type.

One possible solution is to incorporate the closure type into a hash to store/retreive memo. This will remove all the confusion. Unfortunately it requires us to add a `'static` bound to the memo function, which is not a good constraint to have.
