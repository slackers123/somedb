use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};

use crate::{
    byte_reader::ByteReader,
    entity::Entity,
    entity_meta::EntityMeta,
    id::IdType,
    query::{DbQuery, DbQueryMut},
    storable::Storable,
    type_hash::TypeHash,
};

/// This timeout is used for a lot of internal stuff it is pretty arbitraty right now
const SLEEP_TIME: Duration = Duration::from_millis(10);

/// A counter for the db. this is used to allow for multiple db instances in the same process.
static DB_CNT: AtomicU32 = AtomicU32::new(0);

/// A SomeDb instance
#[derive(Debug)]
pub struct Database {
    db_dir: PathBuf,
    stored_types: HashMap<TypeHash, ()>,
    db_id: u32,
}

impl Database {
    pub fn default() -> DbResult<Self> {
        Self::new(PathBuf::from("sdb/"), false)
    }

    pub fn new(db_dir: impl AsRef<Path>, clear: bool) -> DbResult<Self> {
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

                Some((type_hash, ()))
            })
            .collect();

        let db_id = DB_CNT.fetch_add(1, Ordering::Relaxed);

        Ok(Database {
            db_dir: db_dir.as_ref().to_path_buf(),
            stored_types,
            db_id,
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
                .any(|e| e.get_id() == data.get_id())
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

    pub fn write_all<T: Entity>(&mut self, entities: Vec<T>) -> DbResult<()> {
        let type_hash = T::type_hash();

        if !self.stored_types.contains_key(&type_hash) {
            self.add_new_type::<T>()?;
        }

        let last_id = entities
            .last()
            .map(|e| e.get_id())
            .unwrap_or_else(<T::Id as IdType>::initial);

        self.raw_write_all(EntityMeta { last_id, entities })?;

        Ok(())
    }

    pub fn raw_write_all<T: Entity>(&mut self, raw: EntityMeta<T>) -> DbResult<()> {
        let new_data = raw.encoded();

        self.get_wlock::<T>().get()?.write(&new_data)?;

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
        let mut vec = Vec::new();
        self.get_rlock::<T>().get()?.read_to_end(&mut vec)?;

        let mut reader = ByteReader::new(&vec);

        EntityMeta::decoded(reader.reader_for_block())
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
        raw.entities.retain(|e| e.get_id() != id);

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

    /// Creates a [DbQuery](crate::query::DbQuery) which can
    /// be used to query the database like any other iterator.
    pub fn query<T: Entity>(&self) -> DbResult<DbQuery<T>> {
        DbQuery::new(self)
    }

    /// Creates a [DbQueryMut](crate::query::DbQueryMut) which
    /// can be used to query the database and make changes to it
    /// which can then be applied to the database via the
    /// [save_to_db](crate::query::DbIterator::save_to_db) function.
    pub fn query_mut<'a, T: 'static + Entity>(&'a mut self) -> DbResult<DbQueryMut<'a, T>> {
        DbQueryMut::new(self)
    }

    ///////////// LOCKING AND SYNC CODE /////////////

    fn get_rlock<T: Entity>(&self) -> RLock {
        RLock::new(self.type_hash_file_path(&T::type_hash()), self.guid())
    }

    fn get_wlock<T: Entity>(&self) -> WLock {
        WLock::new(self.type_hash_file_path(&T::type_hash()), self.guid())
    }

    fn guid(&self) -> String {
        format!("{}-{}", std::process::id(), self.db_id)
    }
}

fn someone_has_rlock(file: &PathBuf) -> bool {
    let files = fs::read_dir(file.parent().unwrap()).unwrap();

    for file in files {
        if file
            .unwrap()
            .path()
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("rlock")
        {
            return true;
        }
    }
    return false;
}

fn someone_has_wlock(file: &PathBuf) -> bool {
    let files = fs::read_dir(file.parent().unwrap()).unwrap();

    for entry in files {
        if entry.unwrap().path() == file.with_extension("wlock") {
            return true;
        }
    }
    return false;
}

fn rlock_file(file: &PathBuf, guid: &str) -> PathBuf {
    file.with_extension(format!("{guid}-rlock"))
}

pub struct RLock {
    file: PathBuf,
    guid: String,
}

impl RLock {
    pub fn new(file: PathBuf, guid: String) -> Self {
        while someone_has_wlock(&file) {
            std::thread::sleep(SLEEP_TIME);
        }

        fs::write(rlock_file(&file, &guid), "").unwrap();
        RLock { guid, file }
    }

    pub fn get(&self) -> io::Result<File> {
        File::open(&self.file)
    }
}

impl Drop for RLock {
    fn drop(&mut self) {
        fs::remove_file(rlock_file(&self.file, &self.guid)).unwrap();
    }
}

pub struct WLock {
    file: PathBuf,
}

impl WLock {
    pub fn new(file: PathBuf, guid: String) -> Self {
        while someone_has_wlock(&file) {
            std::thread::sleep(SLEEP_TIME);
        }

        fs::write(file.with_extension("wlock"), &guid).unwrap();

        while someone_has_rlock(&file) {
            std::thread::sleep(SLEEP_TIME);
        }

        let lockfile = fs::read_to_string(file.with_extension("wlock")).unwrap();

        if lockfile != guid {
            panic!("there has been some sort of collision");
        }

        WLock { file }
    }

    pub fn get(&self) -> io::Result<File> {
        OpenOptions::new().read(true).write(true).open(&self.file)
    }
}

impl Drop for WLock {
    fn drop(&mut self) {
        fs::remove_file(self.file.with_extension("wlock")).unwrap();
    }
}

pub type DbResult<T> = Result<T, DbError>;

#[derive(Debug)]
pub enum DbError {
    IdExists,
    TypeNotFound,
    IdNotFound,
    IoError(std::io::Error),
    LoadError,
    InvalidFileVersion,
}

impl PartialEq for DbError {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::IdExists => matches!(other, Self::IdExists),
            Self::TypeNotFound => matches!(other, Self::TypeNotFound),
            Self::IdNotFound => matches!(other, Self::IdNotFound),
            Self::IoError(_) => false,
            Self::LoadError => matches!(other, Self::LoadError),
            Self::InvalidFileVersion => matches!(other, Self::InvalidFileVersion),
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
