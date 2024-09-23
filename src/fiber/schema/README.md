# Schema for Fiber Network Messages

The messages type definitions in Fiber Network Protocol (FNP) depend on message type definitions in CKB.
We copied v0.114.0 of upstream schema files in directory [ckb/util/gen-types/schemas](https://github.com/nervosnetwork/ckb/tree/pkg/v0.114.0/util/gen-types/schemas).
One problem is that if we directly generate all type definitions, rust compiler would believe we have two different types of `Transaction`. Because they are from different crates. Instead, we make sure we generate the same code, and directly use the crate [`ckb-gen-types`](https://crates.io/crates/ckb-gen-types). This way we are using the same `Transaction` type. But we must be careful when generating FNP message definitions and importing `ckb-gen-types` (both code should be generated from an identical version of molecule and identical schema files). The schema files in current repo are copied from ckb `v0.114.0`. And as of `ckb-gen-types` v0.114.0, these files in `ckb-gen-types` are created with molecule `v0.7.5`.