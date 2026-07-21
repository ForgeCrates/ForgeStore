- **`storage_engine/`** — Persistent metadata storage layer that wraps RocksDB operations.
- **`raft/`** — Implements Raft consensus for strongly consistent metadata replication.
- **`bucket_metadata/`** — Manages bucket creation, deletion, configuration, and metadata.
- **`object_metadata/`** — Stores and manages metadata for every object in the system.
- **`object_index/`** — Maintains indexes for fast object lookup and listing.
- **`transactions/`** — Coordinates atomic metadata operations across multiple components.
- **`placement/`** — Decides which storage nodes should hold an object's chunks.
- **`namespace/`** — Manages bucket/object naming, paths, and namespace validation.
- **`snapshot/`** — Creates and restores Raft state machine snapshots for faster recovery.
- **`gc/`** — Cleans up orphaned metadata, expired uploads, and old versions.
- **`replication/`** — Handles metadata replication policies beyond core Raft (e.g., geo-replication).
- **`api.rs`** — Exposes the metadata service API used by the gateway and internal services.
- **`main.rs`** — Starts the metadata server, initializes RocksDB, Raft, gRPC, and background workers.


No need of versioning 



object can be public (visible for all) and private (visible to bucket owner)



bucket can have multiple roles (
    owner (all) ,reader (reads), writer (uploads)
)


object that is transfered in chunks will not be accessible at all until fully commited



# RocksDB Metadata Schema

## Key Prefixes

| Prefix | Key | Value | Purpose |
|--------|-----|-------|---------|
| `BUCKET` | `BUCKET:{bucket_id}` | Bucket metadata | Stores bucket information |
| `BUCKET_NAME` | `BUCKET_NAME:{user_id}:{bucket_name}` | `bucket_id` | Maps bucket names to bucket IDs |
| `UBKT` | `UBKT:{user_id}:{bucket_id}` | `1` | Lists buckets owned by a user |
| `BROLE` | `BROLE:{bucket_id}:{user_id}` | `owner`, `reader`, `writer` | User permissions for a bucket |
| `USERROLE` | `USERROLE:{user_id}:{bucket_id}` | `owner`, `reader`, `writer` | Reverse index to list buckets accessible by a user |
| `OBJ` | `OBJ:{object_id}` | Object metadata | Stores object metadata |
| `BOBJ` | `BOBJ:{bucket_id}:{object_name}` | `object_id` | Maps object names to object IDs |
| `PUB` | `PUB:{bucket_id}:{object_name}` | `object_id` | Public object index |
| `OSHARE` | `OSHARE:{object_id}:{user_id}` | Permission | Object-level permissions |
| `USHARE` | `USHARE:{user_id}:{object_id}` | `1` | Reverse object-sharing index |
| `UPLOAD` | `UPLOAD:{upload_id}` | Multipart upload metadata | Tracks multipart uploads |
| `PART` | `PART:{upload_id}:{part_number}` | Part metadata | Multipart upload parts |
| `CHUNK` | `CHUNK:{chunk_id}` | Chunk metadata | Metadata for stored chunks |
| `PLACEMENT` | `PLACEMENT:{object_id}` | Placement metadata | Object → chunk/storage node mapping |

> **Note:** Versioning is not implemented.

---

# Metadata Operations

## 1. Create Bucket

```text
├── RocksDB
│
│      GET key=BUCKET_NAME:{user_id}:{bucket_name} value={bucket_id}
│
│      PUT key=BUCKET:{bucket_id} value={BucketMetadata}
│
│      PUT key=BUCKET_NAME:{user_id}:{bucket_name} value={bucket_id}
│
│      PUT key=UBKT:{user_id}:{bucket_id} value=1
│
│      PUT key=BROLE:{bucket_id}:{user_id} value=owner
│
│      PUT key=USERROLE:{user_id}:{bucket_id} value=owner
```

---

## 2. Delete Bucket

```text
├── RocksDB
│
│      GET key=BUCKET_NAME:{user_id}:{bucket_name} value={bucket_id}
│
│      ITERATE prefix=BOBJ:{bucket_id}:
│
│      DELETE key=BUCKET:{bucket_id}
│
│      DELETE key=BUCKET_NAME:{user_id}:{bucket_name}
│
│      DELETE key=UBKT:{user_id}:{bucket_id}
│
│      DELETE key=BROLE:{bucket_id}:{user_id}
│
│      DELETE key=USERROLE:{user_id}:{bucket_id}
```

---

## 3. Get Bucket Metadata

```text
├── RocksDB
│
│      GET key=BUCKET:{bucket_id} value={BucketMetadata}
```

---

## 4. List My Buckets

```text
├── RocksDB
│
│      ITERATE prefix=UBKT:{user_id}:
│
│      GET key=BUCKET:{bucket_id} value={BucketMetadata}
```

---

## 5. List Buckets Shared With Me

```text
├── RocksDB
│
│      ITERATE prefix=USERROLE:{user_id}:
│
│      GET key=BUCKET:{bucket_id} value={BucketMetadata}
```

---

## 6. Grant Bucket Permission

```text
├── RocksDB
│
│      GET key=BROLE:{bucket_id}:{owner_id} value=owner
│
│      PUT key=BROLE:{bucket_id}:{target_user_id} value=reader
│
│      PUT key=USERROLE:{target_user_id}:{bucket_id} value=reader
```

---

## 7. Revoke Bucket Permission

```text
├── RocksDB
│
│      GET key=BROLE:{bucket_id}:{owner_id} value=owner
│
│      DELETE key=BROLE:{bucket_id}:{target_user_id}
│
│      DELETE key=USERROLE:{target_user_id}:{bucket_id}
```

---

## 8. Upload Object

```text
├── RocksDB
│
│      GET key=BROLE:{bucket_id}:{user_id} value={permission}
│
│      GET key=BOBJ:{bucket_id}:{object_name} value={object_id}
│
│      PUT key=OBJ:{object_id} value={ObjectMetadata}
│
│      PUT key=BOBJ:{bucket_id}:{object_name} value={object_id}
│
│      PUT key=PLACEMENT:{object_id} value={PlacementMetadata}
│
│      PUT key=PUB:{bucket_id}:{object_name} value={object_id}
│      (only if public)
```

---

## 9. Read Object

```text
├── RocksDB
│
│      GET key=BOBJ:{bucket_id}:{object_name} value={object_id}
│
│      GET key=OBJ:{object_id} value={ObjectMetadata}
│
│      GET key=BROLE:{bucket_id}:{user_id} value={permission}
│      (or OSHARE if object-level sharing)
│
│      GET key=PLACEMENT:{object_id} value={PlacementMetadata}
```

---

## 10. Download Object

```text
├── RocksDB
│
│      GET key=PLACEMENT:{object_id} value={PlacementMetadata}
```

---

## 11. Delete Object

```text
├── RocksDB
│
│      GET key=BROLE:{bucket_id}:{user_id} value={permission}
│
│      GET key=BOBJ:{bucket_id}:{object_name} value={object_id}
│
│      GET key=OBJ:{object_id} value={ObjectMetadata}
│
│      DELETE key=OBJ:{object_id}
│
│      DELETE key=BOBJ:{bucket_id}:{object_name}
│
│      DELETE key=PLACEMENT:{object_id}
│
│      DELETE key=PUB:{bucket_id}:{object_name}
```

---

## 12. Rename Object

```text
├── RocksDB
│
│      GET key=BROLE:{bucket_id}:{user_id} value={permission}
│
│      GET key=BOBJ:{bucket_id}:{old_name} value={object_id}
│
│      GET key=OBJ:{object_id} value={ObjectMetadata}
│
│      PUT key=OBJ:{object_id} value={UpdatedObjectMetadata}
│
│      DELETE key=BOBJ:{bucket_id}:{old_name}
│
│      PUT key=BOBJ:{bucket_id}:{new_name} value={object_id}
│
│      DELETE key=PUB:{bucket_id}:{old_name}
│
│      PUT key=PUB:{bucket_id}:{new_name} value={object_id}
│      (if public)
```

---

## 13. Copy Object

```text
├── RocksDB
│
│      GET key=BOBJ:{bucket_id}:{source_name} value={object_id}
│
│      GET key=OBJ:{object_id} value={ObjectMetadata}
│
│      PUT key=OBJ:{new_object_id} value={CopiedObjectMetadata}
│
│      PUT key=BOBJ:{bucket_id}:{destination_name} value={new_object_id}
│
│      PUT key=PLACEMENT:{new_object_id} value={PlacementMetadata}
│
│      PUT key=PUB:{bucket_id}:{destination_name} value={new_object_id}
│      (if public)
```

---

## 14. Move Object Between Buckets

```text
├── RocksDB
│
│      GET key=BROLE:{source_bucket}:{user_id} value={permission}
│
│      GET key=BROLE:{destination_bucket}:{user_id} value={permission}
│
│      GET key=BOBJ:{source_bucket}:{object_name} value={object_id}
│
│      GET key=OBJ:{object_id} value={ObjectMetadata}
│
│      PUT key=OBJ:{object_id} value={UpdatedObjectMetadata}
│
│      DELETE key=BOBJ:{source_bucket}:{object_name}
│
│      PUT key=BOBJ:{destination_bucket}:{object_name} value={object_id}
│
│      DELETE key=PUB:{source_bucket}:{object_name}
│
│      PUT key=PUB:{destination_bucket}:{object_name} value={object_id}
│      (if public)
```

---

## 15. List Objects

```text
├── RocksDB
│
│      GET key=BROLE:{bucket_id}:{user_id} value={permission}
│
│      ITERATE prefix=BOBJ:{bucket_id}:
│
│      GET key=OBJ:{object_id} value={ObjectMetadata}
```

---

## 16. List Public Objects

```text
├── RocksDB
│
│      ITERATE prefix=PUB:{bucket_id}:
│
│      GET key=OBJ:{object_id} value={ObjectMetadata}
```

---

## 17. Make Object Public

```text
├── RocksDB
│
│      GET key=OBJ:{object_id} value={ObjectMetadata}
│
│      PUT key=OBJ:{object_id} value={UpdatedObjectMetadata}
│
│      PUT key=PUB:{bucket_id}:{object_name} value={object_id}
```

---

## 18. Make Object Private

```text
├── RocksDB
│
│      GET key=OBJ:{object_id} value={ObjectMetadata}
│
│      PUT key=OBJ:{object_id} value={UpdatedObjectMetadata}
│
│      DELETE key=PUB:{bucket_id}:{object_name}
```

---

## 19. Start Multipart Upload

```text
├── RocksDB
│
│      GET key=BROLE:{bucket_id}:{user_id} value={permission}
│
│      PUT key=UPLOAD:{upload_id} value={UploadMetadata}
```

---

## 20. Upload Multipart Part

```text
├── RocksDB
│
│      GET key=UPLOAD:{upload_id} value={UploadMetadata}
│
│      PUT key=PART:{upload_id}:{part_number} value={PartMetadata}
```

---

## 21. Complete Multipart Upload

```text
├── RocksDB
│
│      GET key=UPLOAD:{upload_id} value={UploadMetadata}
│
│      ITERATE prefix=PART:{upload_id}:
│
│      PUT key=OBJ:{object_id} value={ObjectMetadata}
│
│      PUT key=BOBJ:{bucket_id}:{object_name} value={object_id}
│
│      PUT key=PLACEMENT:{object_id} value={PlacementMetadata}
│
│      DELETE key=PART:{upload_id}:1
│      DELETE key=PART:{upload_id}:2
│      DELETE key=PART:{upload_id}:3
│
│      DELETE key=UPLOAD:{upload_id}
```
````
