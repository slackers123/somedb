use crate::{
    db::{Database, DbResult},
    entity::Entity,
    entity_meta::EntityMeta,
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

    fn get_db(&mut self) -> &mut Database {
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
    fn get_db(&mut self) -> &mut Database;

    fn filter<P: FnMut(&Self::Item) -> bool>(self, predicate: P) -> DbFilter<Self, P> {
        DbFilter {
            inner: self,
            predicate,
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
        let db = self.get_db();
        db.raw_write_all::<Self::Item>(EntityMeta { last_id, entities })?;

        Ok(())
    }
}

pub struct DbFilter<I, P>
where
    I: DbIterator,
    P: FnMut(&I::Item) -> bool,
{
    inner: I,
    predicate: P,
}

impl<I, P> DbIterator for DbFilter<I, P>
where
    I: DbIterator,
    P: FnMut(&I::Item) -> bool,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(inner_next) = self.inner.next() {
            if (self.predicate)(&inner_next) {
                return Some(inner_next);
            }
        }
        return None;
    }

    fn get_db(&mut self) -> &mut Database {
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

    fn get_db(&mut self) -> &mut Database {
        self.inner.get_db()
    }

    fn get_last_id(&self) -> <Self::Item as Entity>::Id {
        self.inner.get_last_id()
    }
}
