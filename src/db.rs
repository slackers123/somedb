use std::{
    collections::HashMap,
    error::Error,
    fs::{self},
    path::PathBuf,
};

use crate::{byte_reader::ByteReader, entity::Entity, storable::Storable, type_hash::TypeHash};

/// A (for now) in memory db
#[derive(Debug)]
pub struct Database {
    db_dir: PathBuf,
    stored_types: HashMap<TypeHash, ()>,
}

impl Database {
    pub fn default() -> std::io::Result<Self> {
        Self::new(PathBuf::from("sdb/"))
    }

    pub fn new(db_dir: PathBuf) -> std::io::Result<Self> {
        let _ = fs::create_dir_all(&db_dir);

        let stored_types: HashMap<_, _> = fs::read_dir(&db_dir)?
            .filter_map(|f| {
                let name = f.as_ref().ok()?.file_name().into_string().ok()?;
                let parts: Vec<_> = name.split('.').collect();
                if parts.len() > 2 || parts[1] != "sdb" {
                    return None;
                }

                let hash_raw = hex::decode(parts[0]).ok()?;

                if hash_raw.len() != 32 {
                    return None;
                }

                let type_hash = unsafe { TypeHash::from_raw(hash_raw[0..32].try_into().unwrap()) };

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

    pub fn store<T: Entity>(&mut self, data: T) -> DbResult<T> {
        let type_hash = T::type_hash();

        if !self.stored_types.contains_key(&type_hash) {
            self.add_new_type(type_hash)?;
        }

        let mut existing = self.read_all::<T>()?;

        if existing
            .iter()
            .find(|e| e.get_id() == data.get_id())
            .is_some()
        {
            return Err(DbError::IdExists);
        }

        existing.push(data.clone());

        self.raw_write_all(existing)?;

        Ok(data)
    }

    pub fn write_all<T: Entity>(&mut self, all_entities: Vec<T>) -> DbResult<()> {
        let type_hash = T::type_hash();

        if !self.stored_types.contains_key(&type_hash) {
            self.add_new_type(type_hash)?;
        }

        self.raw_write_all(all_entities)?;

        Ok(())
    }

    pub fn raw_write_all<T: Entity>(&mut self, all_entities: Vec<T>) -> DbResult<()> {
        let type_hash = T::type_hash();

        let new_data = all_entities.encoded();

        fs::write(self.type_hash_file_path(&type_hash), new_data)?;

        Ok(())
    }

    pub fn read_all<T: Entity>(&self) -> DbResult<Vec<T>> {
        let type_hash = T::type_hash();

        self.stored_types
            .get(&type_hash)
            .ok_or(DbError::TypeNotFound)?;

        let vec = fs::read(self.type_hash_file_path(&type_hash))?;

        let mut reader = ByteReader::new(&vec);

        Ok(Vec::decoded(reader.reader_for_block()))
    }

    pub fn read_all_ids<T: Entity>(&self) -> DbResult<Vec<T::Id>> {
        Ok(self.read_all::<T>()?.iter().map(|e| e.get_id()).collect())
    }

    pub fn find_by_id<T: Entity>(&self, id: T::Id) -> DbResult<Option<T>> {
        Ok(self.read_all::<T>()?.into_iter().find(|e| e.get_id() == id))
    }

    pub fn update_entity<T: Entity>(&mut self, entity: T) -> DbResult<()> {
        let mut arr = self.read_all::<T>()?;
        let res = arr
            .iter_mut()
            .find(|e| e.get_id() == entity.get_id())
            .ok_or(DbError::IdNotFound)?;

        *res = entity;

        self.write_all(arr)?;

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

    fn add_new_type(&mut self, type_hash: TypeHash) -> Result<(), std::io::Error> {
        let mut new_file = self.db_dir.clone();
        new_file.push(PathBuf::from(format!(
            "{}.sdb",
            hex::encode(type_hash.as_bytes())
        )));

        let mut opts = fs::OpenOptions::new();
        opts.read(true);
        opts.write(true);
        opts.create(true);
        opts.open(new_file)?;

        self.stored_types.insert(type_hash, ());
        Ok(())
    }

    fn type_hash_file_path(&self, type_hash: &TypeHash) -> PathBuf {
        let mut path = self.db_dir.clone();
        path.push(PathBuf::from(format!(
            "{}.sdb",
            hex::encode(type_hash.as_bytes())
        )));
        path
    }
}

type DbResult<T> = Result<T, DbError>;

#[derive(Debug)]
pub enum DbError {
    IdExists,
    TypeNotFound,
    IdNotFound,
    IoError(std::io::Error),
    LoadError(),
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
