<img width="1402" height="1122" alt="image" src="https://github.com/user-attachments/assets/4710ca3f-3e57-4e90-b041-0be8c841164f" />


# Deployable Services

## 1. Gateway
- Exposes the S3-compatible HTTP API.
- Authenticates users (SigV4, access keys).
- Validates requests.
- Handles bucket and object operations.
- Manages multipart uploads.
- Applies rate limiting.
- Routes requests to internal services.

---

## 2. Proxy
- Entry point for internal request routing.
- Chooses the correct storage nodes.
- Performs consistent-hash lookup.
- Load balances requests.
- Retries failed requests.
- Schedules requests across nodes.

---

## 3. Metadata Service
- Stores bucket metadata.
- Stores object metadata.
- Tracks object versions.
- Maintains object locations.
- Runs Raft consensus.
- Coordinates distributed transactions.
- Makes placement decisions.
- Provides strongly consistent metadata.

---

## 4. Storage Node
- Stores actual object data.
- Stores chunks on local disks.
- Performs NVMe/io_uring I/O.
- Compresses data.
- Encrypts data.
- Calculates checksums.
- Reads and writes object chunks.
- Maintains local cache.

---

## 5. Cluster Manager
- Tracks cluster membership.
- Detects failed nodes.
- Exchanges gossip information.
- Sends heartbeats.
- Schedules maintenance tasks.
- Performs failover.
- Balances cluster load.

---

# Background Services

## 6. Rebalancer
- Detects topology changes.
- Plans data migration.
- Moves chunks between nodes.
- Balances storage usage.
- Preserves redundancy during migration.

---

## 7. Scrubber
- Periodically scans stored data.
- Verifies checksums.
- Detects corruption.
- Repairs damaged chunks.
- Maintains long-term data integrity.

---

## 8. Garbage Collector
- Removes orphaned chunks.
- Cleans abandoned multipart uploads.
- Deletes expired object versions.
- Frees unused disk space.
- Reclaims stale metadata.

---

# Library Crates

## IAM
- User management.
- Role management.
- Policy evaluation.
- Access key management.
- Temporary credentials (STS).
- Authorization logic.

---

## Placement
- Consistent hashing.
- Chooses storage nodes.
- Rack awareness.
- Zone awareness.
- Placement policy implementation.

---

## Erasure
- Reed–Solomon encoding.
- Reed–Solomon decoding.
- Stripe creation.
- Parity generation.
- Data recovery after failures.

---

## Replication
- Synchronous replication.
- Asynchronous replication.
- Geo-replication.
- Replica synchronization logic.

---

## Protocol
- Shared network protocols.
- gRPC definitions.
- Protobuf messages.
- S3 request/response models.
- Serialization and deserialization.

---

## Common
- Shared configuration.
- Error types.
- Utility functions.
- Logging helpers.
- Metrics utilities.
- Cryptography helpers.
- Common data types.
- Networking utilities.

---

# Overall Request Flow

1. Client sends a request to **Gateway**.
2. Gateway authenticates and validates the request.
3. Gateway forwards the request to **Proxy**.
4. Proxy consults **Metadata Service** to locate the object.
5. Metadata Service uses the **Placement** library to determine the appropriate storage nodes.
6. Proxy sends the request to the selected **Storage Node(s)**.
7. Storage Nodes use the **Erasure**, **Replication**, **Protocol**, and **Common** library crates while reading or writing data.
8. **Cluster Manager** continuously monitors the health and membership of the cluster.
9. **Rebalancer**, **Scrubber**, and **Garbage Collector** run in the background to maintain cluster health, integrity, and storage efficiency.
