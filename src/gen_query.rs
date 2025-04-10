//! Polars inspired querying

use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Add, BitOr, Div, Mul, Rem, Shl, Shr, Sub},
    time::Instant,
};

macro_rules! int_func_impl {
    ($name:ident, $op:ident, $opop:ident) => {
        fn $name<B>(self, rhs: B) -> BinExpr<E, $opop<E, Self::Output, Self, B>, Self, B>
        where
            Self::Output: $op<Output = Self::Output>,
            B: GenExpr<E, Output = Self::Output>,
        {
            BinExpr {
                a: self,
                b: rhs,
                _int: PhantomData,
            }
        }
    };
}

pub trait GenExpr<E: Debug>: Sized + Debug {
    type Output: Debug;

    fn exec(&self, entity: &E) -> Self::Output;

    fn eq<B: GenExpr<E>>(self, rhs: B) -> BinExpr<E, EqOp<E, Self, B>, Self, B>
    where
        Self::Output: PartialEq<B::Output>,
        B::Output: PartialEq<Self::Output>,
    {
        BinExpr {
            a: self,
            b: rhs,
            _int: PhantomData,
        }
    }

    int_func_impl!(add, Add, AddOp);
    int_func_impl!(sub, Sub, SubOp);
    int_func_impl!(mul, Mul, MulOp);
    int_func_impl!(div, Div, DivOp);
    int_func_impl!(rem, Rem, RemOp);
    int_func_impl!(shl, Shl, ShlOp);
    int_func_impl!(shr, Shr, ShrOp);
}

pub trait BinOp<E: Debug>: Debug {
    type Output: Debug;
    type Lhs: GenExpr<E>;
    type Rhs: GenExpr<E>;
    fn exec(lhs: &Self::Lhs, rhs: &Self::Rhs, entity: &E) -> Self::Output;
}

#[derive(Debug)]
pub struct BinExpr<E: Debug, O, A, B>
where
    O: BinOp<E, Lhs = A, Rhs = B>,
    A: GenExpr<E>,
    B: GenExpr<E>,
{
    a: A,
    b: B,
    _int: PhantomData<(E, O)>,
}

impl<E: Debug, O: BinOp<E, Lhs = A, Rhs = B>, A: GenExpr<E>, B: GenExpr<E>> GenExpr<E>
    for BinExpr<E, O, A, B>
{
    type Output = O::Output;

    fn exec(&self, entity: &E) -> Self::Output {
        O::exec(&self.a, &self.b, entity)
    }
}

#[derive(Debug)]
pub struct EqOp<E: Debug, A, B>
where
    A: GenExpr<E>,
    B: GenExpr<E>,
    A::Output: PartialEq<B::Output>,
    B::Output: PartialEq<A::Output>,
{
    _int: PhantomData<(E, A, B)>,
}

impl<E: Debug, A, B> BinOp<E> for EqOp<E, A, B>
where
    A: GenExpr<E>,
    B: GenExpr<E>,
    A::Output: PartialEq<B::Output>,
    B::Output: PartialEq<A::Output>,
{
    type Lhs = A;
    type Rhs = B;
    type Output = bool;
    fn exec(lhs: &Self::Lhs, rhs: &Self::Rhs, entity: &E) -> Self::Output {
        lhs.exec(entity) == rhs.exec(entity)
    }
}

pub struct NeqOp;

#[derive(Debug)]
pub struct LOrOp<E: Debug, A, B>
where
    A: GenExpr<E, Output = bool>,
    B: GenExpr<E, Output = bool>,
{
    _int: PhantomData<(E, A, B)>,
}

impl<E: Debug, A, B> BinOp<E> for LOrOp<E, A, B>
where
    A: GenExpr<E, Output = bool>,
    B: GenExpr<E, Output = bool>,
{
    type Lhs = A;
    type Rhs = B;
    type Output = bool;
    fn exec(lhs: &Self::Lhs, rhs: &Self::Rhs, entity: &E) -> Self::Output {
        lhs.exec(entity) || rhs.exec(entity)
    }
}

#[derive(Debug)]
pub struct OrOp<E: Debug, A, B>
where
    A: GenExpr<E>,
    B: GenExpr<E>,
{
    _int: PhantomData<(E, A, B)>,
}

impl<E: Debug, A, B> BinOp<E> for OrOp<E, A, B>
where
    A: GenExpr<E>,
    B: GenExpr<E>,
    A::Output: BitOr<B::Output, Output = A::Output>,
    B::Output: BitOr<A::Output, Output = A::Output>,
{
    type Lhs = A;
    type Rhs = B;
    type Output = A::Output;

    fn exec(lhs: &Self::Lhs, rhs: &Self::Rhs, entity: &E) -> Self::Output {
        lhs.exec(entity) | rhs.exec(entity)
    }
}

pub struct AndOp;
pub struct XorOp;

macro_rules! int_op_impl {
    ($name:ident, $op:ident, $calc:tt) => {
        #[derive(Debug)]
        pub struct $name<E, T, A, B>
        where
            E: Debug,
            T: $op<Output = T> + Debug,
            A: GenExpr<E, Output = T>,
            B: GenExpr<E, Output = T>,
        {
            _int: PhantomData<(E, T, A, B)>,
        }

        impl<E, T, A, B> BinOp<E> for $name <E, T, A, B>
        where
            E: Debug,
            T: $op<Output = T> + Debug,
            A: GenExpr<E, Output = T>,
            B: GenExpr<E, Output = T>,
        {
            type Lhs = A;
            type Rhs = B;
            type Output = T;
            fn exec(lhs: &Self::Lhs, rhs: &Self::Rhs, entity: &E) -> Self::Output {
                lhs.exec(entity) $calc rhs.exec(entity)
            }
        }
    };
}

int_op_impl!(AddOp, Add, +);
int_op_impl!(SubOp, Sub, -);
int_op_impl!(MulOp, Mul, *);
int_op_impl!(DivOp, Div, /);
int_op_impl!(RemOp, Rem, %);
int_op_impl!(ShlOp, Shl, <<);
int_op_impl!(ShrOp, Shr, >>);

// int_op_impl!(LtOp, PartialOrd, <);
// int_op_impl!(GtOp, PartialOrd, >);
// int_op_impl!(LteOp, PartialOrd, <=);
// int_op_impl!(GteOp, PartialOrd, >=);

#[derive(Debug)]
pub struct EqExpr<E: Debug, A, B>
where
    A: GenExpr<E>,
    B: GenExpr<E>,
    A::Output: PartialEq<B::Output>,
{
    a: A,
    b: B,
    _int: PhantomData<E>,
}

impl<E: Debug, A, B> GenExpr<E> for EqExpr<E, A, B>
where
    A: GenExpr<E>,
    B: GenExpr<E>,
    A::Output: PartialEq<B::Output>,
{
    type Output = bool;

    fn exec(&self, entity: &E) -> Self::Output {
        self.a.exec(entity) == self.b.exec(entity)
    }
}

pub trait ResolveAttrExpr<T: Debug>: Sized + Debug {
    fn resolve(&self, expr: &AttrExpr<Self, T>) -> T;
}

#[derive(Debug)]
pub struct AttrExpr<E: Debug, T: Debug> {
    _int: PhantomData<(E, T)>,
}

impl<E: ResolveAttrExpr<T> + Debug, T: Debug> GenExpr<E> for AttrExpr<E, T> {
    type Output = T;
    fn exec(&self, entity: &E) -> Self::Output {
        entity.resolve(self)
    }
}

impl<E: Debug, T: Copy + Debug> GenExpr<E> for T {
    type Output = T;
    fn exec(&self, _entity: &E) -> Self::Output {
        *self
    }
}

#[derive(Debug)]
pub struct SomeEntity {
    pub id: i64,
    pub foo: String,
}

impl ResolveAttrExpr<i64> for SomeEntity {
    fn resolve(&self, _expr: &AttrExpr<Self, i64>) -> i64 {
        return self.id;
    }
}

impl ResolveAttrExpr<String> for SomeEntity {
    fn resolve(&self, _expr: &AttrExpr<Self, String>) -> String {
        return self.foo.clone();
    }
}

pub struct ExprEntity<E> {
    _int: PhantomData<E>,
}

impl<E: Debug> ExprEntity<E> {
    pub fn id(&self) -> AttrExpr<E, i64> {
        AttrExpr { _int: PhantomData }
    }

    pub fn foo(&self) -> AttrExpr<E, String> {
        AttrExpr { _int: PhantomData }
    }
}

pub struct QueryBuilder {}

pub fn filter<E: Debug, Q: GenExpr<E, Output = bool>, P: Fn(ExprEntity<E>) -> Q>(
    pattern: P,
    db: &mut ExDb<E>,
) -> QueryBuilder {
    let query = pattern(ExprEntity { _int: PhantomData });

    let start = Instant::now();

    let mut sum = 0;
    for e in &mut db.data {
        let res = query.exec(e);
        sum += res as i32;
    }

    println!(
        "running {} iterations: {:?}, {}",
        db.data.len(),
        start.elapsed(),
        sum
    );

    QueryBuilder {}
}

#[derive(Debug, Default)]
pub struct ExDb<E> {
    pub data: Vec<E>,
}

#[cfg(test)]
mod tests {
    use super::GenExpr;
    use super::{ExDb, SomeEntity, filter};

    #[test]
    fn run_something() {
        let mut db = ExDb {
            data: vec![SomeEntity {
                id: 10,
                foo: "Hello".into(),
            }],
        };

        filter(|e| e.id().eq(e.id()), &mut db);
    }
}
