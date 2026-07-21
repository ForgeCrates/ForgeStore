use crate::storage_engine::MetadataDB;


// ├── RocksDB
// │
// │      GET key=BROLE:{bucket_id}:{user_id} value={permission}
// │
// │      GET key=BOBJ:{bucket_id}:{object_name} value={object_id}
// │
// │      PUT key=OBJ:{object_id} value={ObjectMetadata}
// │
// │      PUT key=BOBJ:{bucket_id}:{object_name} value={object_id}
// │
// │      PUT key=PLACEMENT:{object_id} value={PlacementMetadata}
// │
// │      PUT key=PUB:{bucket_id}:{object_name} value={object_id}
// │      (only if public)


pub fn create_object_metadata(
    db: &MetadataDB,
    bucket_name: &str,
    object_name: &str,
    creator_id: &str,
    placementMetadata: &PlacementMetadata,


) -> Result<(), anyhow::Error> {
    let db = db.inner();
    let bucket_id_key = format!("BUCKET_NAME:{}:{}", creator_id, bucket_name);
    let bucket_id = match db.get(bucket_id_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => anyhow::bail!("Bucket not found"),
    };
    let role_key = format!("BROLE:{}:{}", bucket_id, creator_id);
    match db.get(role_key.as_bytes())? {
        Some(role) => {
            if role != b"owner" && role != b"writer" {
                anyhow::bail!("User does not have permission to create object");
            }
        }
        None => anyhow::bail!("User does not have permission to create object"),
    }
    let object_id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();

    let mut batch = WriteBatch::default();
    batch.put(
        format!("OBJ:{}", object_id),
        serde_json::to_vec(&ObjectMetadata {
            object_id: object_id.to_string(),
            bucket_id: bucket_id.to_string(),
            object_name: object_name.to_string(),
            creator_id: creator_id.to_string(),
            created_at: created_at.to_string(),
        })?,
    );
    batch.put(
        format!("BOBJ:{}:{}", bucket_id, object_name),
        object_id,
    );
    batch.put(
        format!("PLACEMENT:{}", object_id),
        serde_json::to_vec(placementMetadata)?,
    );
    batch.put(
        format!("PUB:{}:{}", bucket_id, object_name),
        object_id,
    );

    db.write(batch)?;
    Ok(())
}



// ├── RocksDB
// │
// │      GET key=BOBJ:{bucket_id}:{object_name} value={object_id}
// │
// │      GET key=OBJ:{object_id} value={ObjectMetadata}
// │
// │      GET key=BROLE:{bucket_id}:{user_id} value={permission}
// │      (or OSHARE if object-level sharing)
// │
// │      GET key=PLACEMENT:{object_id} value={PlacementMetadata}

// TAKE A LOOK AT IT NOT PROPERLY IMPLEMENTED

pub fn get_object_metadata(
    db: &MetadataDB,
    bucket_name: &str,
    object_name: &str,
    user_id: &str,
) -> Result<ObjectMetadata, anyhow::Error> {
    let db = db.inner();
    let bucket_id_key = format!("BUCKET_NAME:{}:{}", user_id, bucket_name);
    let bucket_id = match db.get(bucket_id_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => anyhow::bail!("Bucket not found"),
    };
    let object_id_key = format!("BOBJ:{}:{}", bucket_id, object_name);
    let object_id = match db.get(object_id_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => anyhow::bail!("Object not found"),
    };
    let object_metadata_key = format!("OBJ:{}", object_id);
    let object_metadata = match db.get(object_metadata_key.as_bytes())? {
        Some(metadata) => serde_json::from_slice::<ObjectMetadata>(&metadata)?,
        None => anyhow::bail!("Object metadata not found"),
    };
    
    let role_key = format!("BROLE:{}:{}", bucket_id, user_id);
    match db.get(role_key.as_bytes())? {
        Some(role) => {
            if role != b"owner" && role != b"reader" && role != b"writer" {
                let object_share_key = format!("OSHARE:{}:{}", object_id, user_id);
                match db.get(object_share_key.as_bytes())? {
                    Some(share) => {
                        if share != b"reader" && share != b"writer" {
                            anyhow::bail!("User does not have permission to access object");
                        }
                    }
                    None => anyhow::bail!("User does not have permission to access object"),
                }
            }
        }
        None => anyhow::bail!("User does not have permission to access object"),
    }
   


    let placement_metadata_key = format!("PLACEMENT:{}", object_id);
    let placement_metadata = match db.get(placement_metadata_key.as_bytes())? {
        Some(metadata) => serde_json::from_slice::<PlacementMetadata>(&metadata)?,
        None => anyhow::bail!("Placement metadata not found"),
    };


    Ok(object_metadata)
}




// │      GET key=PLACEMENT:{object_id} value={PlacementMetadata}

pub fn get_placement_metadata(
    db: &MetadataDB,
    object_id: &str,
) -> Result<PlacementMetadata, anyhow::Error> {
    let db = db.inner();
    let placement_metadata_key = format!("PLACEMENT:{}", object_id);
    let placement_metadata = match db.get(placement_metadata_key.as_bytes())? {
        Some(metadata) => serde_json::from_slice::<PlacementMetadata>(&metadata)?,
        None => anyhow::bail!("Placement metadata not found"),
    };
    Ok(placement_metadata)
}


// ├── RocksDB
// │
// │      GET key=BROLE:{bucket_id}:{user_id} value={permission}
// │
// │      GET key=BOBJ:{bucket_id}:{object_name} value={object_id}
// │
// │      GET key=OBJ:{object_id} value={ObjectMetadata}
// │
// │      DELETE key=OBJ:{object_id}
// │
// │      DELETE key=BOBJ:{bucket_id}:{object_name}
// │
// │      DELETE key=PLACEMENT:{object_id}
// │
// │      DELETE key=PUB:{bucket_id}:{object_name}

pub fn delete_object_metadata(
    db: &MetadataDB,
    bucket_name: &str,
    object_name: &str,
    user_id: &str,
) -> Result<(), anyhow::Error> {
    let db = db.inner();
    let bucket_id_key = format!("BUCKET_NAME:{}:{}", user_id, bucket_name);
    let bucket_id = match db.get(bucket_id_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => anyhow::bail!("Bucket not found"),
    };

    let role_key = format!("BROLE:{}:{}", bucket_id, user_id);
    match db.get(role_key.as_bytes())? {
        Some(role) => {
            if role != b"owner" && role != b"writer" {
                anyhow::bail!("User does not have permission to delete object");
            }
        }
        None => anyhow::bail!("User does not have permission to delete object"),
    }

    let object_id_key = format!("BOBJ:{}:{}", bucket_id, object_name);
    let object_id = match db.get(object_id_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => anyhow::bail!("Object not found"),
    };

    let mut batch = WriteBatch::default();
    batch.delete(format!("OBJ:{}", object_id));
    batch.delete(format!("BOBJ:{}:{}", bucket_id, object_name));
    batch.delete(format!("PLACEMENT:{}", object_id));
    batch.delete(format!("PUB:{}:{}", bucket_id, object_name));
    db.write(batch)?;
    Ok(())
}




// ├── RocksDB
// │
// │      GET key=BROLE:{bucket_id}:{user_id} value={permission}
// │
// │      GET key=BOBJ:{bucket_id}:{old_name} value={object_id}
// │
// │      GET key=OBJ:{object_id} value={ObjectMetadata}
// │
// │      PUT key=OBJ:{object_id} value={UpdatedObjectMetadata}
// │
// │      DELETE key=BOBJ:{bucket_id}:{old_name}
// │
// │      PUT key=BOBJ:{bucket_id}:{new_name} value={object_id}
// │
// │      DELETE key=PUB:{bucket_id}:{old_name}
// │
// │      PUT key=PUB:{bucket_id}:{new_name} value={object_id}
// │      (if public)


pub fn rename_object_metadata(
    db: &MetadataDB,
    bucket_name: &str,
    old_name: &str,
    new_name: &str,
    user_id: &str,
) -> Result<(), anyhow::Error> {
    let db = db.inner();
    let bucket_id_key = format!("BUCKET_NAME:{}:{}", user_id, bucket_name);
    let bucket_id = match db.get(bucket_id_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => anyhow::bail!("Bucket not found"),
    };

    let role_key = format!("BROLE:{}:{}", bucket_id, user_id);
    match db.get(role_key.as_bytes())? {
        Some(role) => {
            if role != b"owner" && role != b"writer" {
                anyhow::bail!("User does not have permission to rename object");
            }
        }
        None => anyhow::bail!("User does not have permission to rename object"),
    }

    let object_id_key = format!("BOBJ:{}:{}", bucket_id, old_name);
    let object_id = match db.get(object_id_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => anyhow::bail!("Object not found"),
    };

    let mut batch = WriteBatch::default();
    batch.delete(format!("BOBJ:{}:{}", bucket_id, old_name));
    batch.put(
        format!("BOBJ:{}:{}", bucket_id, new_name),
        object_id.clone(),
    );
    let object_metadata_key = format!("OBJ:{}", object_id);
    let mut object_metadata = match db.get(object_metadata_key.as_bytes())? {
        Some(metadata) => serde_json::from_slice::<ObjectMetadata>(&metadata)?,
        None => anyhow::bail!("Object metadata not found"),
    };
    object_metadata.object_name = new_name.to_string();
    batch.put(
        format!("OBJ:{}", object_id),
        serde_json::to_vec(&object_metadata)?,
    );
    batch.delete(format!("PUB:{}:{}", bucket_id, old_name));
    batch.put(
        format!("PUB:{}:{}", bucket_id, new_name),
        object_id,
    );
    db.write(batch)?;
    Ok(())
}

// ├── RocksDB
// │
// │      GET key=BOBJ:{bucket_id}:{source_name} value={object_id}
// │
// │      GET key=OBJ:{object_id} value={ObjectMetadata}
// │
// │      PUT key=OBJ:{new_object_id} value={CopiedObjectMetadata}
// │
// │      PUT key=BOBJ:{bucket_id}:{destination_name} value={new_object_id}
// │
// │      PUT key=PLACEMENT:{new_object_id} value={PlacementMetadata}
// │
// │      PUT key=PUB:{bucket_id}:{destination_name} value={new_object_id}
// │      (if public)
// LOOKS UNNECESSARY

pub fn copy_object_metadata(
    db: &MetadataDB,
    bucket_name: &str,
    source_name: &str,
    destination_name: &str,
    user_id: &str,
) -> Result<(), anyhow::Error> {
    let db = db.inner();
    let bucket_id_key = format!("BUCKET_NAME:{}:{}", user_id, bucket_name);
    let bucket_id = match db.get(bucket_id_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => anyhow::bail!("Bucket not found"),
    };

    let role_key = format!("BROLE:{}:{}", bucket_id, user_id);
    match db.get(role_key.as_bytes())? {
        Some(role) => {
            if role != b"owner" && role != b"writer" {
                anyhow::bail!("User does not have permission to copy object");
            }
        }
        None => anyhow::bail!("User does not have permission to copy object"),
    }

    let source_object_id_key = format!("BOBJ:{}:{}", bucket_id, source_name);
    let source_object_id = match db.get(source_object_id_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => anyhow::bail!("Source object not found"),
    };

    let source_object_metadata_key = format!("OBJ:{}", source_object_id);
    let source_object_metadata = match db.get(source_object_metadata_key.as_bytes())? {
        Some(metadata) => serde_json::from_slice::<ObjectMetadata>(&metadata)?,
        None => anyhow::bail!("Source object metadata not found"),
    };

    let new_object_id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();

    let mut batch = WriteBatch::default();
    batch.put(
        format!("OBJ:{}", new_object_id),
        serde_json::to_vec(&ObjectMetadata {
            object_id: new_object_id.to_string(),
            bucket_id: bucket_id.to_string(),
            object_name: destination_name.to_string(),
            creator_id: user_id.to_string(),
            created_at: created_at.to_string(),
        })?,
    );
    batch.put(
        format!("BOBJ:{}:{}", bucket_id, destination_name),
        new_object_id.clone(),
    );
    let placement_metadata_key = format!("PLACEMENT:{}", source_object_id);
    let placement_metadata = match db.get(placement_metadata_key.as_bytes())? {
        Some(metadata) => serde_json::from_slice::<PlacementMetadata>(&metadata)?,
        None => anyhow::bail!("Placement metadata not found"),
    };
    batch.put(
        format!("PLACEMENT:{}", new_object_id),
        serde_json::to_vec(&placement_metadata)?,
    );
    batch.put(
        format!("PUB:{}:{}", bucket_id, destination_name),
        new_object_id,
    );
    db.write(batch)?;
    Ok(())
}


// ├── RocksDB
// │
// │      GET key=BROLE:{source_bucket}:{user_id} value={permission}
// │
// │      GET key=BROLE:{destination_bucket}:{user_id} value={permission}
// │
// │      GET key=BOBJ:{source_bucket}:{object_name} value={object_id}
// │
// │      GET key=OBJ:{object_id} value={ObjectMetadata}
// │
// │      PUT key=OBJ:{object_id} value={UpdatedObjectMetadata}
// │
// │      DELETE key=BOBJ:{source_bucket}:{object_name}
// │
// │      PUT key=BOBJ:{destination_bucket}:{object_name} value={object_id}
// │
// │      DELETE key=PUB:{source_bucket}:{object_name}
// │
// │      PUT key=PUB:{destination_bucket}:{object_name} value={object_id}
// │      (if public)

pub fn move_object_metadata(
    db: &MetadataDB,
    source_bucket_name: &str,
    destination_bucket_name: &str,
    object_name: &str,
    user_id: &str,
) -> Result<(), anyhow::Error> {
    let db = db.inner();
    let source_bucket_id_key = format!("BUCKET_NAME:{}:{}", user_id, source_bucket_name);
    let source_bucket_id = match db.get(source_bucket_id_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => anyhow::bail!("Source bucket not found"),
    };
    let destination_bucket_id_key = format!("BUCKET_NAME:{}:{}", user_id, destination_bucket_name);
    let destination_bucket_id = match db.get(destination_bucket_id_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => anyhow::bail!("Destination bucket not found"),
    };

    //  check bucket roles
    let source_role_key = format!("BROLE:{}:{}", source_bucket_id, user_id);
    match db.get(source_role_key.as_bytes())? {
        Some(role) => {
            if role != b"owner" && role != b"writer" {
                anyhow::bail!("User does not have permission to move object from source bucket");
            }
        }
        None => anyhow::bail!("User does not have permission to move object from source bucket"),
    }

    let destination_role_key = format!("BROLE:{}:{}", destination_bucket_id, user_id);
    match db.get(destination_role_key.as_bytes())? {
        Some(role) => {
            if role != b"owner" && role != b"writer" {
                anyhow::bail!("User does not have permission to move object to destination bucket");
            }
        }
        None => anyhow::bail!("User does not have permission to move object to destination bucket"),
    }
    
    // check object existence in source bucket

    let source_object_id_key = format!("BOBJ:{}:{}", source_bucket_id, object_name);
    let object_id = match db.get(source_object_id_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => anyhow::bail!("Object not found in source bucket"),
    };

    
    // 
    let mut batch = WriteBatch::default();
    batch.delete(format!("BOBJ:{}:{}", source_bucket_id, object_name));
    batch.put(
        format!("BOBJ:{}:{}", destination_bucket_id, object_name),
        object_id.clone(),
    );
    let object_metadata_key = format!("OBJ:{}", object_id);
    let mut object_metadata = match db.get(object_metadata_key.as_bytes())? {
        Some(metadata) => serde_json::from_slice::<ObjectMetadata>(&metadata)?,
        None => anyhow::bail!("Object metadata not found"),
    };
    object_metadata.bucket_id = destination_bucket_id.clone();
    batch.put(
        format!("OBJ:{}", object_id),
        serde_json::to_vec(&object_metadata)?,
    );
    batch.delete(format!("PUB:{}:{}", source_bucket_id, object_name));
    batch.put(
        format!("PUB:{}:{}", destination_bucket_id, object_name),
        object_id,
    );
    db.write(batch)?;
    Ok(())
}

// ├── RocksDB
// │
// │      GET key=BROLE:{bucket_id}:{user_id} value={permission}
// │
// │      ITERATE prefix=BOBJ:{bucket_id}:
// │
// │      GET key=OBJ:{object_id} value={ObjectMetadata}

pub fn list_objects_in_bucket(
    db: &MetadataDB,
    bucket_name: &str,
    user_id: &str,
) -> Result<Vec<ObjectMetadata>, anyhow::Error> {
    let db = db.inner();
    let bucket_id_key = format!("BUCKET_NAME:{}:{}", user_id, bucket_name);
    let bucket_id = match db.get(bucket_id_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => anyhow::bail!("Bucket not found"),
    };

    let role_key = format!("BROLE:{}:{}", bucket_id, user_id);
    match db.get(role_key.as_bytes())? {
        Some(role) => {
            if role != b"owner" && role != b"reader" && role != b"writer" {
                anyhow::bail!("User does not have permission to list objects in bucket");
            }
        }
        None => anyhow::bail!("User does not have permission to list objects in bucket"),
    }

    let mut objects = Vec::new();
    let prefix = format!("BOBJ:{}:", bucket_id);
    let iter = db.prefix_iterator(prefix.as_bytes());
    for (key, value) in iter {
        let object_id = String::from_utf8(value.to_vec())?;
        let object_metadata_key = format!("OBJ:{}", object_id);
        if let Some(metadata) = db.get(object_metadata_key.as_bytes())? {
            let object_metadata = serde_json::from_slice::<ObjectMetadata>(&metadata)?;
            objects.push(object_metadata);
        }
    }
    Ok(objects)
}


// ├── RocksDB
// │
// │      ITERATE prefix=PUB:{bucket_id}:
// │
// │      GET key=OBJ:{object_id} value={ObjectMetadata}

pub fn get_public_objects_in_bucket(
    db: &MetadataDB,
    bucket_name: &str,
) -> Result<Vec<ObjectMetadata>, anyhow::Error> {
    let db = db.inner();
    let bucket_id_key = format!("BUCKET_NAME:{}:{}", "public", bucket_name);
    let bucket_id = match db.get(bucket_id_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => anyhow::bail!("Bucket not found"),
    };

    let mut objects = Vec::new();
    let prefix = format!("PUB:{}:", bucket_id);
    let iter = db.prefix_iterator(prefix.as_bytes());
    for (key, value) in iter {
        let object_id = String::from_utf8(value.to_vec())?;
        let object_metadata_key = format!("OBJ:{}", object_id);
        if let Some(metadata) = db.get(object_metadata_key.as_bytes())? {
            let object_metadata = serde_json::from_slice::<ObjectMetadata>(&metadata)?;
            objects.push(object_metadata);
        }
    }
    Ok(objects)
}



// ├── RocksDB
// │
// │      GET key=OBJ:{object_id} value={ObjectMetadata}
// │
// │      PUT key=OBJ:{object_id} value={UpdatedObjectMetadata}
// │
// │      PUT key=PUB:{bucket_id}:{object_name} value={object_id}
pub fn make_object_public(
    db: &MetadataDB,
    bucket_name: &str,
    object_name: &str,
    user_id: &str,
) -> Result<(), anyhow::Error> {
    let db = db.inner();
    let bucket_id_key = format!("BUCKET_NAME:{}:{}", user_id, bucket_name);
    let bucket_id = match db.get(bucket_id_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => anyhow::bail!("Bucket not found"),
    };

    let role_key = format!("BROLE:{}:{}", bucket_id, user_id);
    match db.get(role_key.as_bytes())? {
        Some(role) => {
            if role != b"owner" && role != b"writer" {
                anyhow::bail!("User does not have permission to make object public");
            }
        }
        None => anyhow::bail!("User does not have permission to make object public"),
    }

    let object_id_key = format!("BOBJ:{}:{}", bucket_id, object_name);
    let object_id = match db.get(object_id_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => anyhow::bail!("Object not found"),
    };

    let mut batch = WriteBatch::default();
    batch.put(
        format!("PUB:{}:{}", bucket_id, object_name),
        object_id.clone(),
    );
    db.write(batch)?;
    Ok(())
}

// ├── RocksDB
// │
// │      GET key=OBJ:{object_id} value={ObjectMetadata}
// │
// │      PUT key=OBJ:{object_id} value={UpdatedObjectMetadata}
// │
// │      DELETE key=PUB:{bucket_id}:{object_name}

pub fn make_object_private(
    db: &MetadataDB,
    bucket_name: &str,
    object_name: &str,
    user_id: &str,
) -> Result<(), anyhow::Error> {
    let db = db.inner();
    let bucket_id_key = format!("BUCKET_NAME:{}:{}", user_id, bucket_name);
    let bucket_id = match db.get(bucket_id_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => anyhow::bail!("Bucket not found"),
    };

    let role_key = format!("BROLE:{}:{}", bucket_id, user_id);
    match db.get(role_key.as_bytes())? {
        Some(role) => {
            if role != b"owner" && role != b"writer" {
                anyhow::bail!("User does not have permission to make object private");
            }
        }
        None => anyhow::bail!("User does not have permission to make object private"),
    }

    let object_id_key = format!("BOBJ:{}:{}", bucket_id, object_name);
    let object_id = match db.get(object_id_key.as_bytes())? {
        Some(id) => String::from_utf8(id.to_vec())?,
        None => anyhow::bail!("Object not found"),
    };

    let mut batch = WriteBatch::default();
    batch.delete(format!("PUB:{}:{}", bucket_id, object_name));
    db.write(batch)?;
    Ok(())
}

