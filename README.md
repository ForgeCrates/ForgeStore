# ForgeStorage - Distributed S3 Object Storage

A distributed, S3-compatible object storage system written in Rust — built for high-throughput, erasure-coded, horizontally scalable storage with strong metadata consistency via Raft.



<p align="center">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white">
  <img alt="License" src="https://img.shields.io/badge/license-MIT-blue">
  <img alt="Status" src="https://img.shields.io/badge/status-WIP-yellow">
</p>

---

## Overview

**Distributed S3** is a from-scratch implementation of an S3-compatible object store, designed as a horizontally scalable cluster of storage nodes coordinated by a Raft-backed metadata layer. It targets modern NVMe hardware with `io_uring`-based async I/O, uses Reed–Solomon erasure coding for durability without the storage overhead of full replication, and exposes a standard S3 API surface so existing tooling and SDKs work out of the box.

This project is a learning-and-build exercise in distributed systems engineering — covering consensus, consistent hashing, erasure coding, cluster membership/gossip, rebalancing, and low-level storage I/O — implemented as a set of focused Rust crates in a single workspace.

## Architecture

<p align="center">
  <img src="https://github.com/user-attachments/assets/7871d98f-7645-4678-93c8-7687e4968591" alt="Distributed S3 Architecture" width="800"/>
</p>

## Key Features

- **S3-Compatible API** — Bucket and object operations, multipart uploads, and standard auth (SigV4-style) via the `gateway` crate.
- **Raft-Backed Metadata Cluster** — Strongly consistent object/bucket metadata, versioning, namespace management, and transactional writes.
- **Erasure-Coded Storage** — Reed–Solomon erasure coding with configurable stripe/parity for durability at lower overhead than N-way replication.
- **NVMe-Optimized Storage Nodes** — `io_uring`-based async I/O, checksummed chunk storage, compression, encryption at rest, and local caching.
- **Consistent-Hash Placement** — Rack-aware object placement and configurable placement policies across the cluster.
- **Self-Healing Cluster** — Gossip-based membership, heartbeats, automatic failover, background scrubbing, and rebalancing on topology change.
- **Pluggable Replication** — Synchronous, asynchronous, and geo-replication modes.
- **Garbage Collection** — Orphaned chunk, multipart, and old-version cleanup.
- **Multi-Language SDKs** — Native SDKs for Rust, Go, and Python.
- **Observability** — Built-in metrics, tracing, and structured logging across every service.

## Architecture

The system is composed of independently deployable services:

| Component | Responsibility |
|---|---|
| **Gateway** | Public-facing S3 API: auth, routing, multipart uploads, rate limiting |
| **Proxy** | Request scheduling and consistent-hash based load balancing across nodes |
| **Metadata Cluster** | Raft-replicated object/bucket metadata, versioning, transactions, placement decisions |
| **Storage Node** | Owns physical chunk storage, NVMe I/O, checksums, compression, encryption, caching |
| **Erasure** | Reed–Solomon encode/decode, striping, parity, and recovery |
| **IAM** | Users, roles, policies, access keys, and STS-style temporary credentials |
| **Placement** | Consistent hashing, topology and rack awareness, placement policy evaluation |
| **Cluster Manager** | Membership, gossip, heartbeats, scheduling, failover, load balancing |
| **Replication** | Synchronous, asynchronous, and geo-replication of data |
| **Scrubber** | Background checksum verification and repair |
| **Rebalancer** | Plans and executes data migration when the cluster topology changes |
| **Garbage Collector** | Reclaims orphaned chunks, stale multipart uploads, and expired versions |

For a deeper dive, see [`docs/architecture.md`](docs/architecture.md).

## Project Structure

```
distributed-s3/
│
├── Cargo.toml                 # Workspace
├── rust-toolchain.toml
├── docker-compose.yml
├── Makefile
├── README.md
│
├── configs/
│   ├── cluster.toml
│   ├── gateway.toml
│   ├── metadata.toml
│   ├── storage.toml
│   ├── auth.toml
│   └── logging.toml
│
├── deployments/
│   ├── kubernetes/
│   ├── helm/
│   └── terraform/
│
├── docs/
│   ├── architecture.md
│   ├── api.md
│   ├── storage.md
│   ├── metadata.md
│   ├── consistency.md
│   └── erasure-coding.md
│
├── crates/
│
│   #######################################################
│   ## API LAYER
│   #######################################################
│
│   ├── gateway/
│   │   ├── src/
│   │   │   ├── auth/
│   │   │   ├── multipart/
│   │   │   ├── buckets/
│   │   │   ├── objects/
│   │   │   ├── routing/
│   │   │   ├── middleware/
│   │   │   ├── rate_limit/
│   │   │   ├── metrics/
│   │   │   ├── handlers/
│   │   │   ├── server.rs
│   │   │   └── main.rs
│   │   └── Cargo.toml
│
│   #######################################################
│   ## LOAD BALANCER
│   #######################################################
│
│   ├── proxy/
│   │   ├── src/
│   │   │   ├── scheduler.rs
│   │   │   ├── consistent_hash.rs
│   │   │   ├── health.rs
│   │   │   └── main.rs
│
│   #######################################################
│   ## METADATA CLUSTER
│   #######################################################
│
│   ├── metadata/
│   │   ├── src/
│   │   │
│   │   ├── raft/
│   │   ├── object_metadata/
│   │   ├── bucket_metadata/
│   │   ├── object_versioning/
│   │   ├── object_index/
│   │   ├── transactions/
│   │   ├── placement/
│   │   ├── namespace/
│   │   ├── snapshot/
│   │   ├── gc/
│   │   ├── replication/
│   │   ├── storage_engine/
│   │   ├── api.rs
│   │   └── main.rs
│
│   #######################################################
│   ## STORAGE NODE
│   #######################################################
│
│   ├── storage-node/
│   │   ├── src/
│   │   │
│   │   ├── object_store/
│   │   ├── chunk_store/
│   │   ├── allocator/
│   │   ├── nvme/
│   │   ├── io_uring/
│   │   ├── checksum/
│   │   ├── compression/
│   │   ├── encryption/
│   │   ├── cache/
│   │   ├── replication/
│   │   ├── object_reader/
│   │   ├── object_writer/
│   │   ├── scrubber/
│   │   ├── metrics/
│   │   ├── health/
│   │   └── main.rs
│
│   #######################################################
│   ## ERASURE CODING
│   #######################################################
│
│   ├── erasure/
│   │   ├── src/
│   │   │   ├── reed_solomon.rs
│   │   │   ├── stripe.rs
│   │   │   ├── parity.rs
│   │   │   ├── recovery.rs
│   │   │   └── lib.rs
│
│   #######################################################
│   ## IAM
│   #######################################################
│
│   ├── iam/
│   │   ├── src/
│   │   │   ├── users/
│   │   │   ├── roles/
│   │   │   ├── policies/
│   │   │   ├── access_keys/
│   │   │   ├── sts/
│   │   │   └── lib.rs
│
│   #######################################################
│   ## OBJECT PLACEMENT
│   #######################################################
│
│   ├── placement/
│   │   ├── src/
│   │   │   ├── consistent_hash.rs
│   │   │   ├── topology.rs
│   │   │   ├── rack_awareness.rs
│   │   │   ├── placement_policy.rs
│   │   │   └── lib.rs
│
│   #######################################################
│   ## CLUSTER MANAGEMENT
│   #######################################################
│
│   ├── cluster-manager/
│   │   ├── src/
│   │   │   ├── membership/
│   │   │   ├── gossip/
│   │   │   ├── heartbeat/
│   │   │   ├── scheduler/
│   │   │   ├── failover/
│   │   │   ├── balancing/
│   │   │   └── main.rs
│
│   #######################################################
│   ## REPLICATION
│   #######################################################
│
│   ├── replication/
│   │   ├── src/
│   │   │   ├── synchronous/
│   │   │   ├── asynchronous/
│   │   │   ├── geo/
│   │   │   └── lib.rs
│
│   #######################################################
│   ## SCRUBBER
│   #######################################################
│
│   ├── scrubber/
│   │   ├── src/
│   │   │   ├── checksum.rs
│   │   │   ├── repair.rs
│   │   │   └── main.rs
│
│   #######################################################
│   ## REBALANCER
│   #######################################################
│
│   ├── rebalancer/
│   │   ├── src/
│   │   │   ├── planner.rs
│   │   │   ├── migration.rs
│   │   │   ├── scheduler.rs
│   │   │   └── main.rs
│
│   #######################################################
│   ## GARBAGE COLLECTOR
│   #######################################################
│
│   ├── garbage-collector/
│   │   ├── src/
│   │   │   ├── orphan.rs
│   │   │   ├── multipart.rs
│   │   │   ├── versions.rs
│   │   │   └── main.rs
│
│   #######################################################
│   ## MONITORING
│   #######################################################
│
│   ├── metrics/
│   ├── tracing/
│   ├── logging/
│
│   #######################################################
│   ## SDK
│   #######################################################
│
│   ├── sdk-rust/
│   ├── sdk-go/
│   ├── sdk-python/
│
│   #######################################################
│   ## SHARED LIBRARIES
│   #######################################################
│
│   ├── common/
│   │   ├── config/
│   │   ├── errors/
│   │   ├── types/
│   │   ├── network/
│   │   ├── serialization/
│   │   ├── crypto/
│   │   ├── utils/
│   │   └── lib.rs
│
│   ├── protocol/
│   │   ├── grpc/
│   │   ├── protobuf/
│   │   ├── s3/
│   │   └── lib.rs
│
│   └── client/
│
├── tests/
│   ├── integration/
│   ├── cluster/
│   ├── performance/
│   ├── chaos/
│   └── benchmarks/
│
└── scripts/
    ├── bootstrap.sh
    ├── cluster.sh
    ├── benchmark.sh
    └── deploy.sh
```

## Getting Started

### Prerequisites

- Rust (see `rust-toolchain.toml` for the pinned version)
- Docker & Docker Compose
- Linux with kernel support for `io_uring` (5.11+) recommended for storage-node performance

### Build

```bash
git clone https://github.com/<your-org>/distributed-s3.git
cd distributed-s3
cargo build --workspace --release
```

### Run a Local Cluster

```bash
docker-compose up -d
./scripts/bootstrap.sh
```

This spins up a minimal cluster: a gateway, a metadata quorum, and a handful of storage nodes, using the defaults in `configs/`.

### Run Tests

```bash
cargo test --workspace
./scripts/benchmark.sh   # performance/benchmark suite
```

## Configuration

Each service reads its own TOML config from `configs/`:

| File | Purpose |
|---|---|
| `cluster.toml` | Cluster-wide topology and membership defaults |
| `gateway.toml` | API server, rate limits, TLS |
| `metadata.toml` | Raft group settings, snapshotting |
| `storage.toml` | NVMe device paths, chunk size, cache size |
| `auth.toml` | IAM / credential settings |
| `logging.toml` | Log level and output format |

## Documentation

- [Architecture](docs/architecture.md)
- [API Reference](docs/api.md)
- [Storage Engine](docs/storage.md)
- [Metadata & Raft](docs/metadata.md)
- [Consistency Model](docs/consistency.md)
- [Erasure Coding](docs/erasure-coding.md)

## Deployment

Production deployment manifests live under `deployments/`:

- `kubernetes/` — raw manifests
- `helm/` — Helm chart
- `terraform/` — infrastructure-as-code for cloud provisioning

## Roadmap

- [ ] S3 API surface parity (multipart, ACLs, lifecycle policies)
- [ ] Raft metadata cluster with snapshotting and log compaction
- [ ] Reed–Solomon erasure coding end-to-end
- [ ] `io_uring` storage node I/O path
- [ ] Rack-aware placement and rebalancing
- [ ] Chaos test suite
- [ ] Multi-region / geo-replication

## Contributing

Contributions are welcome. Please open an issue to discuss significant changes before submitting a PR, and make sure `cargo test --workspace` and `cargo clippy --workspace` pass.

## License

Licensed under the [MIT License](LICENSE).
