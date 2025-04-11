//! Polars inspired querying

use std::{
    marker::PhantomData,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub},
};

use crate::{db::Database, entity::Entity};

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

pub trait GenExpr<E: Entity>: Sized {
    type Output;

    fn exec(&self, db: &Database) -> Self::Output;

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

    fn neq<B: GenExpr<E>>(self, rhs: B) -> BinExpr<E, NeqOp<E, Self, B>, Self, B>
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

    fn or<B: GenExpr<E>>(self, rhs: B) -> BinExpr<E, OrOp<E, Self, B>, Self, B>
    where
        Self::Output: BitOr<B::Output, Output = Self::Output>,
        B::Output: BitOr<Self::Output, Output = Self::Output>,
    {
        BinExpr {
            a: self,
            b: rhs,
            _int: PhantomData,
        }
    }

    fn and<B: GenExpr<E>>(self, rhs: B) -> BinExpr<E, AndOp<E, Self, B>, Self, B>
    where
        Self::Output: BitAnd<B::Output, Output = Self::Output>,
        B::Output: BitAnd<Self::Output, Output = Self::Output>,
    {
        BinExpr {
            a: self,
            b: rhs,
            _int: PhantomData,
        }
    }

    fn xor<B: GenExpr<E>>(self, rhs: B) -> BinExpr<E, XorOp<E, Self, B>, Self, B>
    where
        Self::Output: BitXor<B::Output, Output = Self::Output>,
        B::Output: BitXor<Self::Output, Output = Self::Output>,
    {
        BinExpr {
            a: self,
            b: rhs,
            _int: PhantomData,
        }
    }

    fn lor<B: GenExpr<E>>(self, rhs: B) -> BinExpr<E, LOrOp<E, Self, B>, Self, B>
    where
        Self: GenExpr<E, Output = bool>,
        B: GenExpr<E, Output = bool>,
    {
        BinExpr {
            a: self,
            b: rhs,
            _int: PhantomData,
        }
    }

    fn land<B: GenExpr<E>>(self, rhs: B) -> BinExpr<E, LAndOp<E, Self, B>, Self, B>
    where
        Self: GenExpr<E, Output = bool>,
        B: GenExpr<E, Output = bool>,
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

pub trait BinOp<E: Entity> {
    type Output;
    type Lhs: GenExpr<E>;
    type Rhs: GenExpr<E>;
    fn exec(lhs: &Self::Lhs, rhs: &Self::Rhs, db: &Database) -> Self::Output;
}

pub struct BinExpr<E: Entity, O, A, B>
where
    O: BinOp<E, Lhs = A, Rhs = B>,
    A: GenExpr<E>,
    B: GenExpr<E>,
{
    a: A,
    b: B,
    _int: PhantomData<(E, O)>,
}

impl<E: Entity, O: BinOp<E, Lhs = A, Rhs = B>, A: GenExpr<E>, B: GenExpr<E>> GenExpr<E>
    for BinExpr<E, O, A, B>
{
    type Output = O::Output;

    fn exec(&self, db: &Database) -> Self::Output {
        O::exec(&self.a, &self.b, db)
    }
}

pub struct EqOp<E: Entity, A, B> {
    _int: PhantomData<(E, A, B)>,
}

impl<E: Entity, A, B> BinOp<E> for EqOp<E, A, B>
where
    A: GenExpr<E>,
    B: GenExpr<E>,
    A::Output: PartialEq<B::Output>,
    B::Output: PartialEq<A::Output>,
{
    type Lhs = A;
    type Rhs = B;
    type Output = bool;
    fn exec(lhs: &Self::Lhs, rhs: &Self::Rhs, db: &Database) -> Self::Output {
        lhs.exec(db) == rhs.exec(db)
    }
}

pub struct NeqOp<E: Entity, A, B> {
    _int: PhantomData<(E, A, B)>,
}

impl<E: Entity, A, B> BinOp<E> for NeqOp<E, A, B>
where
    A: GenExpr<E>,
    B: GenExpr<E>,
    A::Output: PartialEq<B::Output>,
    B::Output: PartialEq<A::Output>,
{
    type Lhs = A;
    type Rhs = B;
    type Output = bool;
    fn exec(lhs: &Self::Lhs, rhs: &Self::Rhs, db: &Database) -> Self::Output {
        lhs.exec(db) != rhs.exec(db)
    }
}

pub struct LOrOp<E: Entity, A, B> {
    _int: PhantomData<(E, A, B)>,
}

impl<E: Entity, A, B> BinOp<E> for LOrOp<E, A, B>
where
    A: GenExpr<E, Output = bool>,
    B: GenExpr<E, Output = bool>,
{
    type Lhs = A;
    type Rhs = B;
    type Output = bool;
    fn exec(lhs: &Self::Lhs, rhs: &Self::Rhs, db: &Database) -> Self::Output {
        lhs.exec(db) || rhs.exec(db)
    }
}

pub struct LAndOp<E: Entity, A, B> {
    _int: PhantomData<(E, A, B)>,
}

impl<E: Entity, A, B> BinOp<E> for LAndOp<E, A, B>
where
    A: GenExpr<E, Output = bool>,
    B: GenExpr<E, Output = bool>,
{
    type Lhs = A;
    type Rhs = B;
    type Output = bool;
    fn exec(lhs: &Self::Lhs, rhs: &Self::Rhs, db: &Database) -> Self::Output {
        lhs.exec(db) && rhs.exec(db)
    }
}

pub struct OrOp<E: Entity, A, B> {
    _int: PhantomData<(E, A, B)>,
}

impl<E: Entity, A, B> BinOp<E> for OrOp<E, A, B>
where
    A: GenExpr<E>,
    B: GenExpr<E>,
    A::Output: BitOr<B::Output, Output = A::Output>,
    B::Output: BitOr<A::Output, Output = A::Output>,
{
    type Lhs = A;
    type Rhs = B;
    type Output = A::Output;

    fn exec(lhs: &Self::Lhs, rhs: &Self::Rhs, db: &Database) -> Self::Output {
        lhs.exec(db) | rhs.exec(db)
    }
}

pub struct AndOp<E: Entity, A, B> {
    _int: PhantomData<(E, A, B)>,
}

impl<E: Entity, A, B> BinOp<E> for AndOp<E, A, B>
where
    A: GenExpr<E>,
    B: GenExpr<E>,
    A::Output: BitAnd<B::Output, Output = A::Output>,
    B::Output: BitAnd<A::Output, Output = A::Output>,
{
    type Lhs = A;
    type Rhs = B;
    type Output = A::Output;

    fn exec(lhs: &Self::Lhs, rhs: &Self::Rhs, db: &Database) -> Self::Output {
        lhs.exec(db) & rhs.exec(db)
    }
}

pub struct XorOp<E: Entity, A, B> {
    _int: PhantomData<(E, A, B)>,
}

impl<E: Entity, A, B> BinOp<E> for XorOp<E, A, B>
where
    A: GenExpr<E>,
    B: GenExpr<E>,
    A::Output: BitXor<B::Output, Output = A::Output>,
    B::Output: BitXor<A::Output, Output = A::Output>,
{
    type Lhs = A;
    type Rhs = B;
    type Output = A::Output;

    fn exec(lhs: &Self::Lhs, rhs: &Self::Rhs, db: &Database) -> Self::Output {
        lhs.exec(db) ^ rhs.exec(db)
    }
}

macro_rules! int_op_impl {
    ($name:ident, $op:ident, $calc:tt) => {
        pub struct $name<E, T, A, B> {
            _int: PhantomData<(E, T, A, B)>,
        }

        impl<E, T, A, B> BinOp<E> for $name <E, T, A, B>
        where
            E: Entity,
            T: $op<Output = T>,
            A: GenExpr<E, Output = T>,
            B: GenExpr<E, Output = T>,
        {
            type Lhs = A;
            type Rhs = B;
            type Output = T;
            fn exec(lhs: &Self::Lhs, rhs: &Self::Rhs, db: &Database) -> Self::Output {
                lhs.exec(db) $calc rhs.exec(db)
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

macro_rules! ord_op_impl {
    ($name:ident, $calc:tt) => {
        pub struct $name<E, T, A, B> {
            _int: PhantomData<(E, T, A, B)>,
        }

        impl<E, T, A, B> BinOp<E> for $name<E, T, A, B>
        where
            E: Entity,
            T: PartialOrd<T>,
            A: GenExpr<E, Output = T>,
            B: GenExpr<E, Output = T>,
        {
            type Lhs = A;
            type Rhs = B;
            type Output = bool;
            fn exec(lhs: &Self::Lhs, rhs: &Self::Rhs, db: &Database) -> Self::Output {
                lhs.exec(db) $calc rhs.exec(db)
            }
        }
    };
}

ord_op_impl!(LtOp, <);
ord_op_impl!(GtOp, >);
ord_op_impl!(LteOp, <=);
ord_op_impl!(GteOp, >=);

pub struct EqExpr<E: Entity, A, B>
where
    A: GenExpr<E>,
    B: GenExpr<E>,
    A::Output: PartialEq<B::Output>,
{
    a: A,
    b: B,
    _int: PhantomData<E>,
}

impl<E: Entity, A, B> GenExpr<E> for EqExpr<E, A, B>
where
    A: GenExpr<E>,
    B: GenExpr<E>,
    A::Output: PartialEq<B::Output>,
{
    type Output = bool;

    fn exec(&self, db: &Database) -> Self::Output {
        self.a.exec(db) == self.b.exec(db)
    }
}

pub trait ResolveAttrExpr<T>: Entity {
    fn resolve(expr: &AttrExpr<Self, T>, db: &Database) -> T;
}

pub struct AttrExpr<E: Entity, T> {
    _int: PhantomData<(E, T)>,
}

impl<E: ResolveAttrExpr<T>, T> GenExpr<E> for AttrExpr<E, T> {
    type Output = T;
    fn exec(&self, db: &Database) -> Self::Output {
        E::resolve(self, db)
    }
}

impl<E: Entity, T: Copy> GenExpr<E> for T {
    type Output = T;
    fn exec(&self, _db: &Database) -> Self::Output {
        *self
    }
}

pub trait ExprEntity<E> {
    fn new() -> Self;
}
