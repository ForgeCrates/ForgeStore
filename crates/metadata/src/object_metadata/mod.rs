use crate::storage_engine::MetadataDB;

pub fn create_object_metadata(db:&MetadataDB, object_key: &str, metadata: &str) -> Result<(), rocksdb::Error> {
    let key = format!("object_metadata:{}", object_key);
    db.inner().put(key.as_bytes(), metadata.as_bytes())?;
    Ok(())
}
pub fn delete_object_metadata(db:&MetadataDB, object_key: &str) -> Result<(), rocksdb::Error> {
    let key = format!("object_metadata:{}", object_key);
    db.inner().delete(key.as_bytes())?;
    Ok(())
}

pub fn get_object_metadata(db:&MetadataDB, object_key: &str) -> Result<Option<String>, rocksdb::Error> {
    let key = format!("object_metadata:{}", object_key);
    let value = db.inner().get(key.as_bytes())?;
    Ok(value.map(|v| String::from_utf8(v.to_vec()).unwrap()))
}

pub fn list_objects(db:&MetadataDB) -> Result<Vec<String>, rocksdb::Error> {
    let mut objects = Vec::new();
    let iter = db.inner().iterator(rocksdb::IteratorMode::Start);
    for (key, _) in iter {
        let key_str = String::from_utf8(key.to_vec()).unwrap();
        if key_str.starts_with("object_metadata:") {
            let object_key = key_str.trim_start_matches("object_metadata:").to_string();
            objects.push(object_key);
        }
    }
    Ok(objects)
}


pub fn