use rocksdb::{DB, Options};

pub struct MetadataDB {
    db: DB,
}

impl MetadataDB {
    pub fn open(path: &str) -> Result<Self, rocksdb::Error> {
        let mut opts = Options::default();
        opts.create_if_missing(true);

        let db = DB::open(&opts, path)?;

        Ok(Self { db })
    }

    pub fn inner(&self) -> &DB {
        &self.db
    }
}