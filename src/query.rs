use std::marker::PhantomData;

use crate::{
    db::{Database, DbResult},
    entity::Entity,
    entity_meta::EntityMeta,
    gen_query::{ExprEntity, GenExpr},
};

pub struct DbQuery<T: Entity> {
    data: EntityMeta<T>,
    index: usize,
}

impl<T: Entity> DbQuery<T> {
    pub(crate) fn new(db: &Database) -> DbResult<Self> {
        Ok(Self {
            data: db.raw_read_all::<T>()?,
            index: 0,
        })
    }
}

impl<T: Entity> Iterator for DbQuery<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.data.entities.len() {
            return None;
        }
        self.index += 1;
        Some(self.data.entities[self.index - 1].clone())
    }
}

pub struct DbQueryMut<'a, T: Entity> {
    db: &'a mut Database,
    data: EntityMeta<T>,
    index: usize,
}

impl<'a, T: 'a + Entity> DbQueryMut<'a, T> {
    pub(crate) fn new(db: &'a mut Database) -> DbResult<Self> {
        Ok(Self {
            data: db.raw_read_all::<T>()?,
            db,
            index: 0,
        })
    }
}

impl<'a, T: 'a + Entity> DbIterator for DbQueryMut<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.data.entities.len() {
            return None;
        }
        self.index += 1;
        Some(self.data.entities[self.index - 1].clone())
    }

    fn get_db_mut(&mut self) -> &mut Database {
        self.db
    }

    fn get_db(&self) -> &Database {
        self.db
    }

    fn get_last_id(&self) -> <Self::Item as Entity>::Id {
        self.data.last_id
    }
}

pub trait DbIterator: Sized {
    type Item: Entity;
    fn next(&mut self) -> Option<Self::Item>;

    fn get_last_id(&self) -> <Self::Item as Entity>::Id;
    fn get_db_mut(&mut self) -> &mut Database;
    fn get_db(&self) -> &Database;

    fn filter<Q, P>(self, predicate: P) -> DbFilter<Q, P, Self>
    where
        Q: GenExpr<Self::Item, Output = bool>,
        P: Fn(&<Self::Item as Entity>::ExprBase) -> Q,
    {
        DbFilter {
            inner: self,
            query: predicate(&<<Self::Item as Entity>::ExprBase as ExprEntity<
                Self::Item,
            >>::new()),
            _int: PhantomData,
        }
    }

    fn map<P: FnMut(Self::Item) -> Self::Item>(self, predicate: P) -> DbMap<Self, P> {
        DbMap {
            inner: self,
            predicate,
        }
    }

    fn collect_vec(mut self) -> Vec<Self::Item> {
        let mut res = vec![];
        while let Some(next) = self.next() {
            res.push(next);
        }
        res
    }

    fn save_to_db(mut self) -> DbResult<()> {
        let mut entities = Vec::new();
        while let Some(e) = self.next() {
            entities.push(e);
        }
        let last_id = self.get_last_id();
        let db = self.get_db_mut();
        db.raw_write_all::<Self::Item>(EntityMeta { last_id, entities })?;

        Ok(())
    }
}

pub struct DbFilter<Q, P, I>
where
    Q: GenExpr<I::Item, Output = bool>,
    P: Fn(&<I::Item as Entity>::ExprBase) -> Q,
    I: DbIterator,
{
    inner: I,
    query: Q,
    _int: PhantomData<P>,
}

impl<Q, P, I> DbIterator for DbFilter<Q, P, I>
where
    Q: GenExpr<I::Item, Output = bool>,
    P: Fn(&<I::Item as Entity>::ExprBase) -> Q,
    I: DbIterator,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(inner_next) = self.inner.next() {
            let db = self.get_db();
            if self.query.exec(db, inner_next.get_id()) {
                return Some(inner_next);
            }
        }
        None
    }

    fn get_db_mut(&mut self) -> &mut Database {
        self.inner.get_db_mut()
    }

    fn get_db(&self) -> &Database {
        self.inner.get_db()
    }

    fn get_last_id(&self) -> <Self::Item as Entity>::Id {
        self.inner.get_last_id()
    }
}

pub struct DbMap<I, P>
where
    I: DbIterator,
    P: FnMut(I::Item) -> I::Item,
{
    inner: I,
    predicate: P,
}

impl<I, P> DbIterator for DbMap<I, P>
where
    I: DbIterator,
    P: FnMut(I::Item) -> I::Item,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        Some((self.predicate)(self.inner.next()?))
    }

    fn get_db_mut(&mut self) -> &mut Database {
        self.inner.get_db_mut()
    }

    fn get_db(&self) -> &Database {
        self.inner.get_db()
    }

    fn get_last_id(&self) -> <Self::Item as Entity>::Id {
        self.inner.get_last_id()
    }
}
