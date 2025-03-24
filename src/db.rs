use std::{
    collections::HashMap,
    error::Error,
    fs::{self},
    path::PathBuf,
    sync::Mutex,
};

use crate::{
    byte_reader::ByteReader, entity::Entity, entity_meta::EntityMeta, id::IdType,
    storable::Storable, type_hash::TypeHash,
};

static DATABASE_CREATED: Mutex<bool> = Mutex::new(false);

/// A SomeDb instance
#[derive(Debug)]
pub struct Database {
    db_dir: PathBuf,
    stored_types: HashMap<TypeHash, ()>,
}

impl Database {
    pub fn default(clear: bool) -> DbResult<Self> {
        Self::new(PathBuf::from("sdb/"), clear)
    }

    pub fn new(db_dir: PathBuf, clear: bool) -> DbResult<Self> {
        // a little bit hackey but the best we'll do for now
        let mut db_created = DATABASE_CREATED.lock().unwrap();
        if *db_created {
            return Err(DbError::DbInstanceExists);
        }
        *db_created = true;

        if clear {
            let _ = fs::remove_dir_all(&db_dir);
        }

        let _ = fs::create_dir_all(&db_dir);

        let stored_types: HashMap<_, _> = fs::read_dir(&db_dir)?
            .filter_map(|f| {
                let name = f.as_ref().ok()?.file_name().into_string().ok()?;
                let parts: Vec<_> = name.split('.').collect();
                if parts.len() > 2 || parts[1] != "sdb" {
                    return None;
                }

                let type_hash = TypeHash::decode(parts[0]);

                let mut opts = fs::OpenOptions::new();
                opts.read(true);
                opts.write(true);
                opts.open(f.ok()?.path()).ok()?;

                return Some((type_hash, ()));
            })
            .collect();

        Ok(Database {
            db_dir,
            stored_types,
        })
    }

    pub fn store<T: Entity>(&mut self, mut data: T) -> DbResult<T> {
        let type_hash = T::type_hash();

        if !self.stored_types.contains_key(&type_hash) {
            self.add_new_type::<T>()?;
        }

        let mut existing = self.raw_read_all::<T>()?;

        if !T::GENERATE_ID
            && existing
                .entities
                .iter()
                .find(|e| e.get_id() == data.get_id())
                .is_some()
        {
            return Err(DbError::IdExists);
        }

        if T::GENERATE_ID {
            data.set_id(<T::Id as IdType>::generate(existing.last_id))
        }

        existing.entities.push(data.clone());
        existing.last_id = data.get_id();

        self.raw_write_all(existing)?;

        Ok(data)
    }

    pub fn write_all<T: Entity>(&mut self, entities: Vec<T>, last_id: T::Id) -> DbResult<()> {
        let type_hash = T::type_hash();

        if !self.stored_types.contains_key(&type_hash) {
            self.add_new_type::<T>()?;
        }

        self.raw_write_all(EntityMeta { last_id, entities })?;

        Ok(())
    }

    pub fn raw_write_all<T: Entity>(&mut self, raw: EntityMeta<T>) -> DbResult<()> {
        let type_hash = T::type_hash();

        let new_data = raw.encoded();

        fs::write(self.type_hash_file_path(&type_hash), new_data)?;

        Ok(())
    }

    pub fn read_all<T: Entity>(&self) -> DbResult<Vec<T>> {
        let type_hash = T::type_hash();

        self.stored_types
            .get(&type_hash)
            .ok_or(DbError::TypeNotFound)?;

        Ok(self.raw_read_all()?.entities)
    }

    pub fn raw_read_all<T: Entity>(&self) -> DbResult<EntityMeta<T>> {
        let type_hash = T::type_hash();

        let vec = fs::read(self.type_hash_file_path(&type_hash))?;

        let mut reader = ByteReader::new(&vec);

        Ok(EntityMeta::decoded(reader.reader_for_block()))
    }

    pub fn read_all_ids<T: Entity>(&self) -> DbResult<Vec<T::Id>> {
        Ok(self.read_all::<T>()?.iter().map(|e| e.get_id()).collect())
    }

    pub fn find_by_id<T: Entity>(&self, id: T::Id) -> DbResult<Option<T>> {
        Ok(self.read_all::<T>()?.into_iter().find(|e| e.get_id() == id))
    }

    pub fn update_entity<T: Entity>(&mut self, entity: T) -> DbResult<()> {
        let type_hash = T::type_hash();

        self.stored_types
            .get(&type_hash)
            .ok_or(DbError::TypeNotFound)?;

        let mut raw = self.raw_read_all::<T>()?;
        let res = raw
            .entities
            .iter_mut()
            .find(|e| e.get_id() == entity.get_id())
            .ok_or(DbError::IdNotFound)?;

        *res = entity;

        self.raw_write_all(raw)?;

        Ok(())
    }

    pub fn delte_entity_by_id<T: Entity>(&mut self, id: T::Id) -> DbResult<()> {
        let type_hash = T::type_hash();

        self.stored_types
            .get(&type_hash)
            .ok_or(DbError::TypeNotFound)?;

        let mut raw = self.raw_read_all::<T>()?;
        raw.entities = raw
            .entities
            .into_iter()
            .filter(|e| e.get_id() != id)
            .collect();

        self.raw_write_all(raw)?;
        Ok(())
    }

    pub fn delete_entity_store<T: Entity>(&mut self) -> DbResult<()> {
        let type_hash = T::type_hash();
        self.stored_types
            .remove(&type_hash)
            .ok_or(DbError::TypeNotFound)?;
        fs::remove_file(self.type_hash_file_path(&type_hash))?;
        Ok(())
    }

    fn add_new_type<T: Entity>(&mut self) -> DbResult<()> {
        let type_hash = T::type_hash();

        let mut opts = fs::OpenOptions::new();
        opts.read(true);
        opts.write(true);
        opts.create(true);
        opts.open(self.type_hash_file_path(&type_hash))?;

        self.raw_write_all::<T>(EntityMeta {
            last_id: <T::Id as IdType>::initial(),
            entities: vec![],
        })?;

        self.stored_types.insert(type_hash, ());
        Ok(())
    }

    fn type_hash_file_path(&self, type_hash: &TypeHash) -> PathBuf {
        let mut path = self.db_dir.clone();
        path.push(PathBuf::from(format!("{}.sdb", type_hash.encode())));
        path
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        *DATABASE_CREATED.lock().unwrap() = false;
    }
}

type DbResult<T> = Result<T, DbError>;

#[derive(Debug)]
pub enum DbError {
    IdExists,
    TypeNotFound,
    IdNotFound,
    IoError(std::io::Error),
    LoadError,
    DbInstanceExists,
}

impl PartialEq for DbError {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::IdExists => match other {
                Self::IdExists => true,
                _ => false,
            },
            Self::TypeNotFound => match other {
                Self::TypeNotFound => true,
                _ => false,
            },
            Self::IdNotFound => match other {
                Self::IdNotFound => true,
                _ => false,
            },
            Self::IoError(_) => false,
            Self::LoadError => match other {
                Self::LoadError => true,
                _ => false,
            },
            Self::DbInstanceExists => match other {
                Self::DbInstanceExists => true,
                _ => false,
            },
        }
    }
}

impl From<std::io::Error> for DbError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for DbError {}
