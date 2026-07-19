have to use protoc + tonic + tonic-build + prost

protocol/
│
├── protobuf/
│   ├── storage.proto
│   ├── metadata.proto
│   ├── placement.proto
│   ├── cluster.proto
│   └── auth.proto
│
├── grpc/
│   ├── storage.rs
│   ├── metadata.rs
│   ├── placement.rs
│   └── mod.rs
│
├── s3/
│   ├── put_object.rs
│   ├── get_object.rs
│   ├── delete_object.rs
│   ├── list_objects.rs
│   ├── multipart.rs
│   └── mod.rs
│
└── lib.rs