
//! Stolen from https://docs.rs/crate/boxfnonce/0.1.0

pub trait FnBox<Arguments, Result> {
    fn call(self: Box<Self>, args: Arguments)
    -> Result;
}

impl <A1, Result, F: FnOnce(A1) -> Result> FnBox<(A1,), Result> for F {
    fn call(self: Box<Self>, (a1,): (A1,)) -> Result {
        let this: Self = *self;
        this(a1)
    }
}

pub struct SendBoxFnOnce<'a, Arguments, Result =
                         ()>(Box<FnBox<Arguments, Result> + Send + 'a>);

impl <'a, Args, Result> SendBoxFnOnce<'a, Args, Result> {
    /// call inner function, consumes the box.
    ///
    /// `call_tuple` can be used if the arguments are available as
    /// tuple. Each usable instance of SendBoxFnOnce<(...), Result> has
    /// a separate `call` method for passing arguments "untupled".
    #[inline]
    pub fn call_tuple(self, args: Args) -> Result { self.0.call(args) }
    /// `SendBoxFnOnce::new` is an alias for `SendBoxFnOnce::from`.
    #[inline]
    pub fn new<F>(func: F) -> Self where Self: From<F> {
        Self::from(func)
    }
}



impl <'a, A1, Result> SendBoxFnOnce<'a, (A1,), Result> {
    /// call inner function, consumes the box.
    #[inline]
    pub fn call(self, a1: A1) -> Result { FnBox::call(self.0, (a1,)) }
}

impl <'a, A1, Result, F: 'a + FnOnce(A1) -> Result + Send> From<F> for
 SendBoxFnOnce<'a, (A1,), Result> {
    fn from(func: F) -> Self {
        SendBoxFnOnce(Box::new(func) as
                          Box<FnBox<(A1,), Result> + Send + 'a>)
    }
}

