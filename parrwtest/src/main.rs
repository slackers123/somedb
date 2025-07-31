use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU32, Ordering},
    thread,
    time::Duration,
};

const SLEEP_TIME: Duration = Duration::from_millis(10);

fn main() {
    let handles = (0..100).map(|i| {
        thread::spawn(move || {
            let mut db = Db::new(PathBuf::from("."));

            db.write(format!("something{}", i));

            println!("{}", db.read());
        })
    });

    for handle in handles {
        handle.join().unwrap();
    }
}

/// A counter for the db. this is used to allow for multiple db instances in the same process.
static DB_CNT: AtomicU32 = AtomicU32::new(0);
pub struct Db {
    path: PathBuf,
    db_id: u32,
}

impl Db {
    pub fn new(path: PathBuf) -> Self {
        let db_id = DB_CNT.fetch_add(1, Ordering::Relaxed);
        Self { path, db_id }
    }

    fn has_wlock(&self) -> bool {
        let files = fs::read_dir(self.path.clone()).unwrap();

        for file in files {
            if file.unwrap().file_name().to_str() == Some("wlock") {
                return true;
            }
        }
        return false;
    }

    fn has_rlock(&self) -> bool {
        let files = fs::read_dir(self.path.clone()).unwrap();

        for file in files {
            if file
                .unwrap()
                .file_name()
                .to_str()
                .unwrap()
                .starts_with("rlock")
            {
                return true;
            }
        }
        return false;
    }

    pub fn read(&self) -> String {
        while self.has_wlock() {
            std::thread::sleep(SLEEP_TIME);
        }

        fs::write(self.path.join(self.rlock_file_name()), "").unwrap();

        let output = fs::read_to_string(self.path.join("data")).unwrap();

        fs::remove_file(self.path.join(self.rlock_file_name())).unwrap();

        return output;
    }

    pub fn write(&mut self, data: String) {
        while self.has_wlock() {
            std::thread::sleep(SLEEP_TIME);
        }

        fs::write(self.path.join("wlock"), self.guid()).unwrap();

        while self.has_rlock() {
            std::thread::sleep(SLEEP_TIME);
        }

        let lockfile = fs::read_to_string(self.path.join("wlock")).unwrap();

        if lockfile != self.guid() {
            panic!("there has been some sort of collision");
        }

        fs::write(self.path.join("data"), data).unwrap();

        fs::remove_file(self.path.join("wlock")).unwrap();
    }

    fn rlock_file_name(&self) -> String {
        format!("rlock-{}", self.guid())
    }

    fn guid(&self) -> String {
        format!("{}-{}", std::process::id(), self.db_id)
    }
}

pub struct RLock {}

impl RLock {
    pub fn new() -> Self {
        RLock {}
    }

    pub fn read(&self) -> String {
        "asdf".into()
    }

    pub fn upgrade(self) -> WLock {
        WLock::new()
    }
}

impl Drop for RLock {
    fn drop(&mut self) {}
}

pub struct WLock {}

impl WLock {
    pub fn new() -> Self {
        WLock {}
    }

    pub fn write(data: String) {}

    pub fn read() -> String {
        "asdf".into()
    }
}

impl Drop for WLock {
    fn drop(&mut self) {}
}
