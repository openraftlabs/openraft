async raft
==========
[![Build Status](https://travis-ci.com/railgun-rs/actix-raft.svg?branch=master)](https://travis-ci.com/railgun-rs/actix-raft)
[![Crates.io](https://img.shields.io/crates/v/actix-raft.svg)](https://crates.io/crates/actix-raft)
[![docs.rs](https://docs.rs/actix-raft/badge.svg)](https://docs.rs/actix-raft)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)
![Crates.io](https://img.shields.io/crates/d/actix-raft.svg)
![Crates.io](https://img.shields.io/crates/dv/actix-raft.svg)
[![GitHub issues open](https://img.shields.io/github/issues-raw/railgun-rs/actix-raft.svg)]()
[![GitHub issues closed](https://img.shields.io/github/issues-closed-raw/railgun-rs/actix-raft.svg)]()

An implementation of the [Raft distributed consensus protocol](https://raft.github.io/) using [the Tokio framework](https://tokio.rs/). Blazing fast Rust, a modern consensus protocol, and a reliable async runtime — this project intends to provide a consensus backbone for the next generation of distributed data storage systems (SQL, NoSQL, KV, Streaming &c). Please ⭐ on [github](https://github.com/railgun-rs/actix-raft)!

[The guide](https://railgun-rs.github.io/actix-raft) is the best place to get started, followed by [the docs](https://docs.rs/actix-raft/latest/actix_raft/) for more in-depth details.

This crate differs from other Raft implementations in that:
- It is fully reactive and embraces the async ecosystem. It is driven by actual Raft related events taking place in the system as opposed to being driven by a `tick` operation. Batching of messages during replication is still used whenever possible for maximum throughput.
- Storage and network integration is well defined via two traits `RaftStorage` & `RaftNetwork`. This provides applications maximum flexibility in being able to choose their storage and networking mediums. See the [storage](https://railgun-rs.github.io/actix-raft/storage.html) & [network](https://railgun-rs.github.io/actix-raft/network.html) chapters of the guide for more details.
- All interaction with the Raft node is well defined via a single public `Raft` type, which is used to spawn the Raft async task, and to interact with that task. The API for this system is clear and concise. See the [raft](https://railgun-rs.github.io/actix-raft/raft.html) chapter in the guide.
- It fully supports dynamic cluster membership changes according to the Raft spec. See the [`dynamic membership`](https://railgun-rs.github.io/actix-raft/dynamic-membership.html) chapter in the guide.
- Details on initial cluster formation, and how to effectively do so from an application level perspective, are discussed in the [cluster formation](https://railgun-rs.github.io/actix-raft/cluster-formation.html) chapter in the guide.
- Automatic log compaction with snapshots, as well as snapshot streaming from the leader node to follower nodes is fully supported and configurable.
- The entire code base is [instrumented with tracing](https://docs.rs/tracing/). This can be used for [standard logging](https://docs.rs/tracing/latest/tracing/index.html#log-compatibility), or for [distributed tracing](https://docs.rs/tracing/latest/tracing/index.html#related-crates), and the verbosity can be [statically configured at compile time](https://docs.rs/tracing/latest/tracing/level_filters/index.html) to completely remove all instrumentation below the configured level.

This implementation strictly adheres to the [Raft spec](https://raft.github.io/raft.pdf) (*pdf warning*), and all data models use the same nomenclature found in the spec for better understandability. This implementation of Raft has integration tests covering all aspects of a Raft cluster's lifecycle including: cluster formation, dynamic membership changes, snapshotting, writing data to a live cluster and more.

If you are building an application using this Raft implementation, open an issue and let me know! I would love to add your project's name & logo to a users list in this project.

### contributing
Check out the [CONTRIBUTING.md](https://github.com/railgun-rs/actix-raft/blob/master/CONTRIBUTING.md) guide for more details on getting started with contributing to this project.

### license
actix-raft is licensed under the terms of the MIT License or the Apache License 2.0, at your choosing.

----

**NOTE:** the appearance of the "section" symbols `§` throughout this project are references to specific sections of the Raft spec.
