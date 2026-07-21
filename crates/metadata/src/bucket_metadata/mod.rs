use crate::storage_engine::MetadataDB;



// ├── RocksDB
// │
// │      GET key=BUCKET_NAME:{user_id}:{bucket_name} value={bucket_id}
// │
// │      PUT key=BUCKET:{bucket_id} value={BucketMetadata}
// │
// │      PUT key=BUCKET_NAME:{user_id}:{bucket_name} value={bucket_id}
// │
// │      PUT key=UBKT:{user_id}:{bucket_id} value=1
// │
// │      PUT key=BROLE:{bucket_id}:{user_id} value=owner
// │
// │      PUT key=USERROLE:{user_id}:{bucket_id} value=owner

pub fn create_bucket_metadata(
    db: &MetadataDB,
    user_id: &str,
    bucket_name: &str,
) -> Result<(), anyhow::Error> {
    let bucket_id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();

    let db = db.inner();
    let bucket_name_key =
        format!("BUCKET_NAME:{}:{}", user_id, bucket_name);
    if db.get(bucket_name_key.as_bytes())?.is_some() {
        anyhow::bail!("Bucket already exists");
    }
    let mut batch = WriteBatch::default();
    batch.put(
        format!("BUCKET:{}", bucket_id),
        serde_json::to_vec(&BucketMetadata {
            bucket_id: bucket_id.to_string(),
            owner_id: user_id.to_string(),
            bucket_name: bucket_name.to_string(),
            created_at: created_at.to_string(),
        })?,
    );
    batch.put(format!("BUCKET_NAME:{}:{}", user_id, bucket_name), bucket_id);
    batch.put(
        format!("UBKT:{}:{}", user_id, bucket_id),
        b"1",
    );
    batch.put(
        format!("BROLE:{}:{}", bucket_id, user_id),
        b"owner",
    );
    batch.put(
        format!("USERROLE:{}:{}", user_id, bucket_id),
        b"owner",
    );
    db.write(batch)?;
    Ok(())
}


// ├── RocksDB
// │
// │      GET key=BUCKET_NAME:{user_id}:{bucket_name} value={bucket_id}
// │
// │      ITERATE prefix=BOBJ:{bucket_id}:
// │
// │      DELETE key=BUCKET:{bucket_id}
// │
// │      DELETE key=BUCKET_NAME:{user_id}:{bucket_name}
// │
// │      DELETE key=UBKT:{user_id}:{bucket_id}
// │
// │      DELETE key=BROLE:{bucket_id}:{user_id}
// │
// │      DELETE key=USERROLE:{user_id}:{bucket_id}

pub fn delete_bucket_metadata(
    db: &MetadataDB,
    user_id: &str,
    bucket_name: &str,
) -> Result<(), anyhow::Error> {

    let db = db.inner();
    let bucket_name_key =
        format!("BUCKET_NAME:{}:{}", user_id, bucket_name);
    let bucket_id = match db.get(bucket_name_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => anyhow::bail!("Bucket not found"),
    };
    let mut batch = WriteBatch::default();
    batch.delete(format!("BUCKET:{}", bucket_id));
    batch.delete(bucket_name_key);
    batch.delete(format!("UBKT:{}:{}", user_id, bucket_id));
    batch.delete(format!("BROLE:{}:{}", bucket_id, user_id));
    batch.delete(format!("USERROLE:{}:{}", user_id, bucket_id));
    db.write(batch)?;
    Ok(())
}



// ├── RocksDB
// │
// │      GET key=BUCKET:{bucket_id} value={BucketMetadata}


pub fn get_bucket_metadata(
    db: &MetadataDB,
    user_id: &str,
    bucket_name: &str,
) -> Result<Option<BucketMetadata>, anyhow::Error> {
    let db = db.inner();
    let bucket_name_key =
        format!("BUCKET_NAME:{}:{}", user_id, bucket_name);
    let bucket_id = match db.get(bucket_name_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => return Ok(None),
    };
    let bucket_metadata_key = format!("BUCKET:{}", bucket_id);
    let bucket_metadata = match db.get(bucket_metadata_key.as_bytes())? {
        Some(data) => serde_json::from_slice(&data)?,
        None => return Ok(None),
    };
    Ok(Some(bucket_metadata))
}



// ├── RocksDB
// │
// │      ITERATE prefix=UBKT:{user_id}:
// │
// │      GET key=BUCKET:{bucket_id} value={BucketMetadata}

pub fn get_user_buckets(
    db: &MetadataDB,
    user_id: &str,
) -> Result<Vec<BucketMetadata>, anyhow::Error> {
    let db = db.inner();
    let prefix = format!("UBKT:{}:", user_id);
    let mut buckets = Vec::new();
    for item in db.prefix_iterator(prefix.as_bytes()) {
        let (key, _) = item?;
        let key_str = String::from_utf8(key.to_vec())?;
        let parts: Vec<&str> = key_str.split(':').collect();
        if parts.len() == 3 {
            let bucket_id = parts[2];
            if let Some(bucket_metadata) =
                get_bucket_metadata(db, user_id, bucket_id)?
            {
                buckets.push(bucket_metadata);
            }
        }
    }
    Ok(buckets)
}


// ├── RocksDB
// │
// │      ITERATE prefix=USERROLE:{user_id}:
// │
// │      GET key=BUCKET:{bucket_id} value={BucketMetadata}


pub fn get_buckets_shared_with_user(
    db: &MetadataDB,
    user_id: &str,
) -> Result<Vec<BucketMetadata>, anyhow::Error> {
    let db = db.inner();
    let prefix = format!("USERROLE:{}:", user_id);
    let mut buckets = Vec::new();
    for item in db.prefix_iterator(prefix.as_bytes()) {
        let (key, _) = item?;
        let key_str = String::from_utf8(key.to_vec())?;
        let parts: Vec<&str> = key_str.split(':').collect();
        if parts.len() == 3 {
            let bucket_id = parts[2];
            if let Some(bucket_metadata) =
                get_bucket_metadata(db, user_id, bucket_id)?
            {
                buckets.push(bucket_metadata);
            }
        }
    }
    Ok(buckets)
}

// ├── RocksDB
// │
// │      GET key=BROLE:{bucket_id}:{owner_id} value=owner
// │
// │      PUT key=BROLE:{bucket_id}:{target_user_id} value=reader
// │
// │      PUT key=USERROLE:{target_user_id}:{bucket_id} value=reader


pub fn share_bucket_with_user(
    db: &MetadataDB,
    owner_id: &str,
    bucket_name: &str,
    target_user_id: &str,
    target_role: &str,
) -> Result<(), anyhow::Error> {
    let db = db.inner();
    let bucket_name_key =
        format!("BUCKET_NAME:{}:{}", owner_id, bucket_name);
    let bucket_id = match db.get(bucket_name_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => anyhow::bail!("Bucket not found"),
    };
    let role_key = format!("BROLE:{}:{}", bucket_id, owner_id);
    match db.get(role_key.as_bytes())? {
        Some(role) => {
            if role != b"owner" {
                anyhow::bail!("User is not the owner of the bucket");
            }
        }
        None => anyhow::bail!("User is not the owner of the bucket"),
    }
    let target_role_key = format!("BROLE:{}:{}", bucket_id, target_user_id);
    db.put(target_role_key.as_bytes(), target_role.as_bytes())?;
    let user_role_key = format!("USERROLE:{}:{}", target_user_id, bucket_id);
    db.put(user_role_key.as_bytes(), target_role.as_bytes())?;
    Ok(())
}


// ├── RocksDB
// │
// │      GET key=BROLE:{bucket_id}:{owner_id} value=owner
// │
// │      DELETE key=BROLE:{bucket_id}:{target_user_id}
// │
// │      DELETE key=USERROLE:{target_user_id}:{bucket_id}
pub fn revoke_bucket_permission(
    db: &MetadataDB,
    owner_id: &str,
    bucket_name: &str,
    target_user_id: &str,
) -> Result<(), anyhow::Error> {
    let db = db.inner();
    let bucket_name_key =
        format!("BUCKET_NAME:{}:{}", owner_id, bucket_name);
    let bucket_id = match db.get(bucket_name_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => anyhow::bail!("Bucket not found"),
    };
    let role_key = format!("BROLE:{}:{}", bucket_id, owner_id);
    match db.get(role_key.as_bytes())? {
        Some(role) => {
            if role != b"owner" {
                anyhow::bail!("User is not the owner of the bucket");
            }
        }
        None => anyhow::bail!("User is not the owner of the bucket"),
    }
    let target_role_key = format!("BROLE:{}:{}", bucket_id, target_user_id);
    db.delete(target_role_key.as_bytes())?;
    let user_role_key = format!("USERROLE:{}:{}", target_user_id, bucket_id);
    db.delete(user_role_key.as_bytes())?;
    Ok(())
}