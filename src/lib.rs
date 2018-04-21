//! The enum [`Either`] with variants `Left` and `Right` is a general purpose
//! sum type with two cases.
//!
//! [`Either`]: enum.Either.html
//!
//! **Crate features:**
//!
//! * `"use_std"`
//! Enabled by default. Disable to make the library `#![no_std]`.
//!
//! * `"serde"`
//! Disabled by default. Enable to `#[derive(Serialize, Deserialize)]` for `Either`
//!

#![doc(html_root_url = "https://docs.rs/either/1/")]

#![cfg_attr(feature = "try_trait", allow(unstable_features))]
#![cfg_attr(feature = "try_trait", feature(try_trait))]

#![cfg_attr(all(not(test), not(feature = "use_std")), no_std)]

#[cfg(all(not(test), not(feature = "use_std")))]
extern crate core as std;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

use std::convert::{AsRef, AsMut};
use std::fmt;
use std::iter;
use std::ops::Deref;
use std::ops::DerefMut;
#[cfg(any(test, feature = "use_std"))]
use std::io::{self, Write, Read, BufRead};
#[cfg(all(feature = "use_std", feature = "try_trait"))]
use std::ops::Try;
#[cfg(any(test, feature = "use_std"))]
use std::error::Error;

pub use Either::{Left, Right};

/// The enum `Either` with variants `Left` and `Right` is a general purpose
/// sum type with two cases.
///
/// The `Either` type is symmetric and treats its variants the same way, without
/// preference.
/// (For representing success or error, use the regular `Result` enum instead.)
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Either<L, R> {
    /// A value of type `L`.
    Left(L),
    /// A value of type `R`.
    Right(R),
}

macro_rules! either {
    ($value:expr, $pattern:pat => $result:expr) => (
        match $value {
            Either::Left($pattern) => $result,
            Either::Right($pattern) => $result,
        }
    )
}

/// Macro for unwrapping the left side of an `Either`, which fails early
/// with the opposite side. Can only be used in functions that return
/// `Either` because of the early return of `Right` that it provides.
///
/// See also `try_right!` for its dual, which applies the same just to the
/// right side.
///
/// # Example
///
/// ```
/// #[macro_use] extern crate either;
/// use either::{Either, Left, Right};
///
/// fn twice(wrapper: Either<u32, &str>) -> Either<u32, &str> {
///     let value = try_left!(wrapper);
///     Left(value * 2)
/// }
///
/// fn main() {
///     assert_eq!(twice(Left(2)), Left(4));
///     assert_eq!(twice(Right("ups")), Right("ups"));
/// }
/// ```
#[macro_export]
macro_rules! try_left {
    ($expr:expr) => (
        match $expr {
            $crate::Left(val) => val,
            $crate::Right(err) => return $crate::Right(::std::convert::From::from(err))
        }
    )
}

/// Dual to `try_left!`, see its documentation for more information.
#[macro_export]
macro_rules! try_right {
    ($expr:expr) => (
        match $expr {
            $crate::Left(err) => return $crate::Left(::std::convert::From::from(err)),
            $crate::Right(val) => val
        }
    )
}

impl<L, R> Either<L, R> {
    /// Return true if the value is the `Left` variant.
    ///
    /// ```
    /// use either::*;
    ///
    /// let values = [Left(1), Right("the right value")];
    /// assert_eq!(values[0].is_left(), true);
    /// assert_eq!(values[1].is_left(), false);
    /// ```
    pub fn is_left(&self) -> bool {
        match *self {
            Left(_) => true,
            Right(_) => false,
        }
    }

    /// Return true if the value is the `Right` variant.
    ///
    /// ```
    /// use either::*;
    ///
    /// let values = [Left(1), Right("the right value")];
    /// assert_eq!(values[0].is_right(), false);
    /// assert_eq!(values[1].is_right(), true);
    /// ```
    pub fn is_right(&self) -> bool {
        !self.is_left()
    }

    /// Convert the left side of `Either<L, R>` to an `Option<L>`.
    ///
    /// ```
    /// use either::*;
    ///
    /// let left: Either<_, ()> = Left("some value");
    /// assert_eq!(left.left(),  Some("some value"));
    ///
    /// let right: Either<(), _> = Right(321);
    /// assert_eq!(right.left(), None);
    /// ```
    pub fn left(self) -> Option<L> {
        match self {
            Left(l) => Some(l),
            Right(_) => None,
        }
    }

    /// Convert the right side of `Either<L, R>` to an `Option<R>`.
    ///
    /// ```
    /// use either::*;
    ///
    /// let left: Either<_, ()> = Left("some value");
    /// assert_eq!(left.right(),  None);
    ///
    /// let right: Either<(), _> = Right(321);
    /// assert_eq!(right.right(), Some(321));
    /// ```
    pub fn right(self) -> Option<R> {
        match self {
            Left(_) => None,
            Right(r) => Some(r),
        }
    }

    /// Convert `&Either<L, R>` to `Either<&L, &R>`.
    ///
    /// ```
    /// use either::*;
    ///
    /// let left: Either<_, ()> = Left("some value");
    /// assert_eq!(left.as_ref(), Left(&"some value"));
    ///
    /// let right: Either<(), _> = Right("some value");
    /// assert_eq!(right.as_ref(), Right(&"some value"));
    /// ```
    pub fn as_ref(&self) -> Either<&L, &R> {
        match *self {
            Left(ref inner) => Left(inner),
            Right(ref inner) => Right(inner),
        }
    }

    /// Convert `&mut Either<L, R>` to `Either<&mut L, &mut R>`.
    ///
    /// ```
    /// use either::*;
    ///
    /// fn mutate_left(value: &mut Either<u32, u32>) {
    ///     if let Some(l) = value.as_mut().left() {
    ///         *l = 999;
    ///     }
    /// }
    ///
    /// let mut left = Left(123);
    /// let mut right = Right(123);
    /// mutate_left(&mut left);
    /// mutate_left(&mut right);
    /// assert_eq!(left, Left(999));
    /// assert_eq!(right, Right(123));
    /// ```
    pub fn as_mut(&mut self) -> Either<&mut L, &mut R> {
        match *self {
            Left(ref mut inner) => Left(inner),
            Right(ref mut inner) => Right(inner),
        }
    }

    /// Convert `Either<L, R>` to `Either<R, L>`.
    ///
    /// ```
    /// use either::*;
    ///
    /// let left: Either<_, ()> = Left(123);
    /// assert_eq!(left.flip(), Right(123));
    ///
    /// let right: Either<(), _> = Right("some value");
    /// assert_eq!(right.flip(), Left("some value"));
    /// ```
    pub fn flip(self) -> Either<R, L> {
        match self {
            Left(l) => Right(l),
            Right(r) => Left(r),
        }
    }

    /// Apply the function `f` on the value in the `Left` variant if it is present rewrapping the
    /// result in `Left`.
    ///
    /// ```
    /// use either::*;
    ///
    /// let left: Either<_, u32> = Left(123);
    /// assert_eq!(left.map_left(|x| x * 2), Left(246));
    ///
    /// let right: Either<u32, _> = Right(123);
    /// assert_eq!(right.map_left(|x| x * 2), Right(123));
    /// ```
    pub fn map_left<F, M>(self, f: F) -> Either<M, R>
        where F: FnOnce(L) -> M
    {
        match self {
            Left(l) => Left(f(l)),
            Right(r) => Right(r),
        }
    }

    /// Apply the function `f` on the value in the `Right` variant if it is present rewrapping the
    /// result in `Right`.
    ///
    /// ```
    /// use either::*;
    ///
    /// let left: Either<_, u32> = Left(123);
    /// assert_eq!(left.map_right(|x| x * 2), Left(123));
    ///
    /// let right: Either<u32, _> = Right(123);
    /// assert_eq!(right.map_right(|x| x * 2), Right(246));
    /// ```
    pub fn map_right<F, S>(self, f: F) -> Either<L, S>
        where F: FnOnce(R) -> S
    {
        match self {
            Left(l) => Left(l),
            Right(r) => Right(f(r)),
        }
    }

    /// Apply one of two functions depending on contents, unifying their result. If the value is
    /// `Left(L)` then the first function `f` is applied; if it is `Right(R)` then the second
    /// function `g` is applied.
    ///
    /// ```
    /// use either::*;
    ///
    /// fn square(n: u32) -> i32 { (n * n) as i32 }
    /// fn negate(n: i32) -> i32 { -n }
    ///
    /// let left: Either<u32, i32> = Left(4);
    /// assert_eq!(left.either(square, negate), 16);
    ///
    /// let right: Either<u32, i32> = Right(-4);
    /// assert_eq!(right.either(square, negate), 4);
    /// ```
    pub fn either<F, G, T>(self, f: F, g: G) -> T
      where F: FnOnce(L) -> T,
            G: FnOnce(R) -> T
    {
        match self {
            Left(l) => f(l),
            Right(r) => g(r),
        }
    }

    /// Like `either`, but provide some context to whichever of the
    /// functions ends up being called.
    ///
    /// ```
    /// // In this example, the context is a mutable reference
    /// use either::*;
    ///
    /// let mut result = Vec::new();
    ///
    /// let values = vec![Left(2), Right(2.7)];
    ///
    /// for value in values {
    ///     value.either_with(&mut result,
    ///                       |ctx, integer| ctx.push(integer),
    ///                       |ctx, real| ctx.push(f64::round(real) as i32));
    /// }
    ///
    /// assert_eq!(result, vec![2, 3]);
    /// ```
    pub fn either_with<Ctx, F, G, T>(self, ctx: Ctx, f: F, g: G) -> T
      where F: FnOnce(Ctx, L) -> T,
            G: FnOnce(Ctx, R) -> T
    {
        match self {
            Left(l) => f(ctx, l),
            Right(r) => g(ctx, r),
        }
    }

    /// Apply the function `f` on the value in the `Left` variant if it is present.
    ///
    /// ```
    /// use either::*;
    ///
    /// let left: Either<_, u32> = Left(123);
    /// assert_eq!(left.left_and_then::<_,()>(|x| Right(x * 2)), Right(246));
    ///
    /// let right: Either<u32, _> = Right(123);
    /// assert_eq!(right.left_and_then(|x| Right::<(), _>(x * 2)), Right(123));
    /// ```
    pub fn left_and_then<F, S>(self, f: F) -> Either<S, R>
        where F: FnOnce(L) -> Either<S, R>
    {
        match self {
            Left(l) => f(l),
            Right(r) => Right(r),
        }
    }

    /// Apply the function `f` on the value in the `Right` variant if it is present.
    ///
    /// ```
    /// use either::*;
    ///
    /// let left: Either<_, u32> = Left(123);
    /// assert_eq!(left.right_and_then(|x| Right(x * 2)), Left(123));
    ///
    /// let right: Either<u32, _> = Right(123);
    /// assert_eq!(right.right_and_then(|x| Right(x * 2)), Right(246));
    /// ```
    pub fn right_and_then<F, S>(self, f: F) -> Either<L, S>
        where F: FnOnce(R) -> Either<L, S>
    {
        match self {
            Left(l) => Left(l),
            Right(r) => f(r),
        }
    }

    /// Convert the inner value to an iterator.
    ///
    /// ```
    /// use either::*;
    ///
    /// let left: Either<_, Vec<u32>> = Left(vec![1, 2, 3, 4, 5]);
    /// let mut right: Either<Vec<u32>, _> = Right(vec![]);
    /// right.extend(left.into_iter());
    /// assert_eq!(right, Right(vec![1, 2, 3, 4, 5]));
    /// ```
    pub fn into_iter(self) -> Either<L::IntoIter, R::IntoIter>
        where L: IntoIterator,
              R: IntoIterator<Item = L::Item>
    {
        match self {
            Left(l) => Left(l.into_iter()),
            Right(r) => Right(r.into_iter()),
        }
    }
}

impl<T, L, R> Either<(T, L), (T, R)> {
    /// Factor out a homogeneous type from an either of pairs.
    ///
    /// Here, the homogeneous type is the first element of the pairs.
    ///
    /// ```
    /// use either::*;
    /// let left: Either<_, (u32, String)> = Left((123, vec![0]));
    /// assert_eq!(left.factor_first().0, 123);
    ///
    /// let right: Either<(u32, Vec<u8>), _> = Right((123, String::new()));
    /// assert_eq!(right.factor_first().0, 123);
    /// ```
    pub fn factor_first(self) -> (T, Either<L, R>) {
        match self {
            Left((t, l)) => (t, Left(l)),
            Right((t, r)) => (t, Right(r)),
        }
    }
}

impl<T, L, R> Either<(L, T), (R, T)> {
    /// Factor out a homogeneous type from an either of pairs.
    ///
    /// Here, the homogeneous type is the second element of the pairs.
    ///
    /// ```
    /// use either::*;
    /// let left: Either<_, (String, u32)> = Left((vec![0], 123));
    /// assert_eq!(left.factor_second().1, 123);
    ///
    /// let right: Either<(Vec<u8>, u32), _> = Right((String::new(), 123));
    /// assert_eq!(right.factor_second().1, 123);
    /// ```
    pub fn factor_second(self) -> (Either<L, R>, T) {
        match self {
            Left((l, t)) => (Left(l), t),
            Right((r, t)) => (Right(r), t),
        }
    }
}

impl<T> Either<T, T> {
    /// Extract the value of an either over two equivalent types.
    ///
    /// ```
    /// use either::*;
    ///
    /// let left: Either<_, u32> = Left(123);
    /// assert_eq!(left.into_inner(), 123);
    ///
    /// let right: Either<u32, _> = Right(123);
    /// assert_eq!(right.into_inner(), 123);
    /// ```
    pub fn into_inner(self) -> T {
        either!(self, inner => inner)
    }
}

/// Convert from `Result` to `Either` with `Ok => Right` and `Err => Left`.
impl<L, R> From<Result<R, L>> for Either<L, R> {
    fn from(r: Result<R, L>) -> Self {
        match r {
            Err(e) => Left(e),
            Ok(o) => Right(o),
        }
    }
}

/// Convert from `Either` to `Result` with `Right => Ok` and `Left => Err`.
impl<L, R> Into<Result<R, L>> for Either<L, R> {
    fn into(self) -> Result<R, L> {
        match self {
            Left(l) => Err(l),
            Right(r) => Ok(r),
        }
    }
}

impl<L, R, A> Extend<A> for Either<L, R>
    where L: Extend<A>, R: Extend<A>
{
    fn extend<T>(&mut self, iter: T)
        where T: IntoIterator<Item=A>
    {
        either!(*self, ref mut inner => inner.extend(iter))
    }
}

/// `Either<L, R>` is an iterator if both `L` and `R` are iterators.
impl<L, R> Iterator for Either<L, R>
    where L: Iterator, R: Iterator<Item=L::Item>
{
    type Item = L::Item;

    fn next(&mut self) -> Option<Self::Item> {
        either!(*self, ref mut inner => inner.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        either!(*self, ref inner => inner.size_hint())
    }

    fn fold<Acc, G>(self, init: Acc, f: G) -> Acc
        where G: FnMut(Acc, Self::Item) -> Acc,
    {
        either!(self, inner => inner.fold(init, f))
    }

    fn count(self) -> usize {
        either!(self, inner => inner.count())
    }

    fn last(self) -> Option<Self::Item> {
        either!(self, inner => inner.last())
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        either!(*self, ref mut inner => inner.nth(n))
    }

    fn collect<B>(self) -> B
        where B: iter::FromIterator<Self::Item>
    {
        either!(self, inner => inner.collect())
    }

    fn all<F>(&mut self, f: F) -> bool
        where F: FnMut(Self::Item) -> bool
    {
        either!(*self, ref mut inner => inner.all(f))
    }
}

impl<L, R> DoubleEndedIterator for Either<L, R>
    where L: DoubleEndedIterator, R: DoubleEndedIterator<Item=L::Item>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        either!(*self, ref mut inner => inner.next_back())
    }
}

impl<L, R> ExactSizeIterator for Either<L, R>
    where L: ExactSizeIterator, R: ExactSizeIterator<Item=L::Item>
{
}

#[cfg(any(test, feature = "use_std"))]
/// `Either<L, R>` implements `Read` if both `L` and `R` do.
///
/// Requires crate feature `"use_std"`
impl<L, R> Read for Either<L, R>
    where L: Read, R: Read
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        either!(*self, ref mut inner => inner.read(buf))
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        either!(*self, ref mut inner => inner.read_to_end(buf))
    }
}

#[cfg(any(test, feature = "use_std"))]
/// Requires crate feature `"use_std"`
impl<L, R> BufRead for Either<L, R>
    where L: BufRead, R: BufRead
{
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        either!(*self, ref mut inner => inner.fill_buf())
    }

    fn consume(&mut self, amt: usize) {
        either!(*self, ref mut inner => inner.consume(amt))
    }
}

#[cfg(any(test, feature = "use_std"))]
/// `Either<L, R>` implements `Write` if both `L` and `R` do.
///
/// Requires crate feature `"use_std"`
impl<L, R> Write for Either<L, R>
    where L: Write, R: Write
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        either!(*self, ref mut inner => inner.write(buf))
    }

    fn flush(&mut self) -> io::Result<()> {
        either!(*self, ref mut inner => inner.flush())
    }
}

impl<L, R, Target> AsRef<Target> for Either<L, R>
    where L: AsRef<Target>, R: AsRef<Target>
{
    fn as_ref(&self) -> &Target {
        either!(*self, ref inner => inner.as_ref())
    }
}

impl<L, R, Target> AsMut<Target> for Either<L, R>
    where L: AsMut<Target>, R: AsMut<Target>
{
    fn as_mut(&mut self) -> &mut Target {
        either!(*self, ref mut inner => inner.as_mut())
    }
}

impl<L, R> Deref for Either<L, R>
    where L: Deref, R: Deref<Target=L::Target>
{
    type Target = L::Target;

    fn deref(&self) -> &Self::Target {
        either!(*self, ref inner => &*inner)
    }
}

impl<L, R> DerefMut for Either<L, R>
    where L: DerefMut, R: DerefMut<Target=L::Target>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        either!(*self, ref mut inner => &mut *inner)
    }
}

#[cfg(all(feature = "use_std", feature = "try_trait"))]
/// Requires crate feature `"use_std"`
impl<L, R> Try for Either<L, R> {
    type Ok = R;
    type Error = L;

    fn into_result(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Left(l) => Err(l),
            Right(r) => Ok(r),
        }
    }

    fn from_error(v: Self::Error) -> Self {
        Left(v)
    }

    fn from_ok(v: Self::Ok) -> Self {
        Right(v)
    }
}

#[cfg(any(test, feature = "use_std"))]
/// `Either` implements `Error` if *both* `L` and `R` implement it.
impl<L, R> Error for Either<L, R>
    where L: Error, R: Error
{
    fn description(&self) -> &str {
        either!(*self, ref inner => inner.description())
    }

    fn cause(&self) -> Option<&Error> {
        either!(*self, ref inner => inner.cause())
    }
}

impl<L, R> fmt::Display for Either<L, R>
    where L: fmt::Display, R: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        either!(*self, ref inner => inner.fmt(f))
    }
}

#[test]
fn basic() {
    let mut e = Left(2);
    let r = Right(2);
    assert_eq!(e, Left(2));
    e = r;
    assert_eq!(e, Right(2));
    assert_eq!(e.left(), None);
    assert_eq!(e.right(), Some(2));
    assert_eq!(e.as_ref().right(), Some(&2));
    assert_eq!(e.as_mut().right(), Some(&mut 2));
}

#[test]
fn macros() {
    fn a() -> Either<u32, u32> {
        let x: u32 = try_left!(Right(1337u32));
        Left(x * 2)
    }
    assert_eq!(a(), Right(1337));

    fn b() -> Either<String, &'static str> {
        Right(try_right!(Left("foo bar")))
    }
    assert_eq!(b(), Left(String::from("foo bar")));
}

#[test]
fn deref() {
    fn is_str(_: &str) {}
    let value: Either<String, &str> = Left(String::from("test"));
    is_str(&*value);
}

#[test]
fn iter() {
    let x = 3;
    let mut iter = match x {
        1...3 => Left(0..10),
        _ => Right(17..),
    };

    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.count(), 9);
}

#[test]
fn read_write() {
    use std::io;

    let use_stdio = false;
    let mockdata = [0xff; 256];

    let mut reader = if use_stdio {
        Left(io::stdin())
    } else {
        Right(&mockdata[..])
    };

    let mut buf = [0u8; 16];
    assert_eq!(reader.read(&mut buf).unwrap(), buf.len());
    assert_eq!(&buf, &mockdata[..buf.len()]);

    let mut mockbuf = [0u8; 256];
    let mut writer = if use_stdio {
        Left(io::stdout())
    } else {
        Right(&mut mockbuf[..])
    };

    let buf = [1u8; 16];
    assert_eq!(writer.write(&buf).unwrap(), buf.len());
}

#[test]
fn error() {
    let invalid_utf8 = b"\xff";
    let res = || -> Result<_, Either<_, _>> {
        try!(::std::str::from_utf8(invalid_utf8).map_err(Left));
        try!("x".parse::<i32>().map_err(Right));
        Ok(())
    }();
    assert!(res.is_err());
    res.unwrap_err().description(); // make sure this can be called
}

#[cfg(feature = "try_trait")]
#[test]
fn try_trait_to_result() {
    fn can_fail(value: Either<i32, &str>) -> Result<&str, i32> {
        Ok(value?)
    }

    assert_eq!(can_fail(Left(42)   ), Err(42) );
    assert_eq!(can_fail(Right("hi")), Ok("hi"));
}

#[cfg(feature = "try_trait")]
#[test]
fn try_trait_to_either() {
    fn can_fail(value: Result<&str, i32>) -> Either<i32, &str> {
        Right(value?)
    }

    assert_eq!(can_fail(Err(42) ), Left(42)   );
    assert_eq!(can_fail(Ok("hi")), Right("hi"));
}
