# CCH (Cross-Chain Hub): Summary and Future Plan

## Overview

CCH is the cross-chain payment bridge in Fiber Network. It enables atomic swaps between CKB (via Fiber payment channels) and Bitcoin (via Lightning Network) using Hash Time-Locked Contracts (HTLCs). A hub operator ("Ingrid" in the spec) runs both a Fiber Network Node and a Lightning node (LND), and facilitates cross-chain payments by locking funds on both sides with the same payment hash.

The CCH module lives in `crates/fiber-lib/src/cch/` with supporting types in `fiber-types` and `fiber-json-types`.

## What Has Been Done

### Phase 1: Bootstrap and Prototype (Mar–Jul 2024)

The CCH module was bootstrapped in late March 2024, starting as a skeleton `ccn` module. Early iterative PRs built out the RPC layer, send/receive order logic, and LND invoice subscription. These were consolidated into a working cross-chain hub demo (PR #46) along with the protocol specification (PR #85, `docs/specs/cross-chain-htlc.md`).

**Key deliverables:**
- Basic `CchActor` with `SendBTC` (CKB→BTC) and `ReceiveBTC` (BTC→CKB) flows
- JSON-RPC endpoints: `send_btc`, `receive_btc`, `get_cch_order`
- LND gRPC integration for hold invoices and payment tracking
- Protocol specification document

### Phase 2: Renames and Infrastructure (Jul–Nov 2024)

The codebase underwent major renaming (ckb→fiber, cfn→fiber). LDK was removed in favor of LND as the sole Lightning backend. Maintenance fixes addressed e2e tests, documentation, and CLI argument handling.

### Phase 3: Multi-Hop and Store Monitoring (Feb–Dec 2025)

This was the most transformative phase. The original CCH only supported direct single-hop TLC manipulation, which was fragile and limiting. After several attempts (PRs #511, #522, #615), the work was split into a clean PR stack:

- **Store change notifications** (PR #930): Infrastructure for recording and notifying store changes, enabling CCH to react to Fiber payment state transitions.
- **Settle payment with invoice** (PR #934): Network actor gained the ability to settle payments by invoice, a prerequisite for CCH's incoming settlement flow.
- **Multi-hop fiber payments** (PR #942): Replaced direct TLC manipulation with standard multi-hop payment routing via `send_payment` / invoices. This was the single biggest architectural improvement.
- **LND tracker refactor** (PR #948): Clean separation of LND gRPC subscription logic into `LndTrackerActor` with concurrent tracking, queue management, and timeouts.
- **Store change monitoring** (PR #950): CCH actor subscribes to Fiber store changes (invoice status, payment session, preimage events) to drive the state machine automatically.

### Phase 4: FSM Refactor, Persistence, and Validation (Nov 2025–Jan 2026)

A stacked PR series (#971→#998→#1045) fundamentally restructured the CCH internals:

- **Finite State Machine** (PR #971): Introduced `CchOrderStateMachine` with deterministic state transitions (`Pending → IncomingAccepted → OutgoingInFlight → OutgoingSuccess → Success`, with `Failed` reachable from any non-final state). Actions (`TrackIncomingInvoice`, `SendOutgoingPayment`, `TrackOutgoingPayment`, `SettleIncomingInvoice`) are dispatched on state entry. Retry with exponential backoff handles transient failures.
- **Expiry and preimage validations** (PR #998): Comprehensive validation of invoice expiry times, preimage-to-hash verification, and the "half for outgoing" expiry safety rule (documented in `docs/specs/cch-expiry-dependency.md`).
- **Order persistence and housekeeping** (PR #1045): Orders are persisted to the store under `CCH_ORDER_PREFIX`. On startup, active orders are resumed and expired orders are marked as failed. A `CchOrderSchedulerActor` handles time-based expiry and 21-day pruning of final orders.

### Phase 5: Bug Fixes and Hardening (Feb 2026)

A concentrated burst of reliability fixes:

- **Fee inclusion in BTC receive** (PR #1142): The hold invoice amount now correctly includes the hub fee.
- **Permanent error detection** (PR #1143): `is_permanent_send_payment_error` identifies non-retryable errors (no route, expired, self-payment, invalid parameter) to fail orders immediately instead of retrying indefinitely.
- **BTC block time overflow** (PR #1145): Extracted `BTC_BLOCK_TIME_MILLIS` constant and guarded against arithmetic overflow when converting blocks to milliseconds.
- **Network/currency validation** (PR #1146): `send_btc` validates that the BTC invoice's network (mainnet/testnet/regtest) matches the node's configured CKB network currency. `receive_btc` validates the CKB invoice currency.
- **Dynamic HTLC expiry check** (PR #1148): Before dispatching the outgoing payment, CCH computes the remaining incoming expiry, allocates half for the outgoing route, and sets `cltv_limit` or `tlc_expiry_limit` accordingly. This prevents fund loss when the outgoing payment routes through multiple hops whose cumulative expiry exceeds the incoming side.

### Phase 6: Separate Service Mode (Mar 2026)

- **Separate service mode** (PR #1165): CCH can now run as a standalone process, connecting to an external Fiber node via HTTP JSON-RPC for order operations and subscribing to store changes over WebSocket with automatic reconnection. Introduced `CchFiberAgentRef` to unify the in-process and RPC backends behind a single interface. Standalone mode requires `wrapped_btc_type_script` in the config since the contracts context is unavailable.
- **Type and field renames** (PR #1211): Naming consistency cleanup across CCH types and fields.

## Current Architecture

```
                    ┌─────────────────────────────────────────────┐
                    │                  CchActor                   │
                    │                                             │
   JSON-RPC ───────►│  SendBTC / ReceiveBTC / GetCchOrder         │
                    │                                             │
                    │  CchOrderStateMachine  ◄── CchTrackingEvent │
                    │         │                       ▲           │
                    │         ▼                       │           │
                    │  ActionDispatcher          LndTrackerActor  │
                    │    │          │                  │           │
                    │    ▼          ▼                  │           │
                    │  CchFiberAgentRef          LND gRPC         │
                    │  (InProcess | Rpc)                           │
                    │         │                                   │
                    └─────────┼───────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼                               ▼
    NetworkActor (in-process)       HTTP RPC (separate mode)
    + Store subscriptions           + WebSocket subscriptions
```

### Key components:
- **`CchActor`**: Central actor handling RPC commands, tracking events, store changes, and action retries.
- **`CchOrderStateMachine`**: Deterministic FSM governing order state transitions.
- **`ActionDispatcher`**: Maps order states to executable actions (track, send, settle).
- **`LndTrackerActor`**: Subscribes to LND via gRPC for invoice and payment status updates.
- **`CchFiberAgentRef`**: Uniform interface to Fiber node (in-process `NetworkActor` or HTTP RPC proxy).
- **`CchOrderSchedulerActor`**: Time-based expiry and pruning of orders.
- **`CchOrderStore`**: Trait for persisting orders, implemented by the main store.

### Deployment modes:
1. **In-process**: Fiber + CKB + CCH in one binary. `CchFiberAgentRef::InProcess` talks to `NetworkActor`; store changes arrive via in-process subscriptions.
2. **Standalone**: CCH-only process. `CchFiberAgentRef::Rpc` talks to Fiber via HTTP; store changes arrive via WebSocket. Requires `fiber_rpc_url` and `wrapped_btc_type_script` in config.

### Order lifecycle:
```
Pending ──► IncomingAccepted ──► OutgoingInFlight ──► OutgoingSuccess ──► Success
  │              │                     │                    │
  └──────────────┴─────────────────────┴────────────────────┘
                              │
                              ▼
                           Failed
```

1. **Pending**: Order created, incoming invoice added. `TrackIncomingInvoice` action started.
2. **IncomingAccepted**: Incoming invoice accepted (funds locked). `SendOutgoingPayment` and `TrackOutgoingPayment` actions started.
3. **OutgoingInFlight**: Outgoing payment dispatched, awaiting settlement.
4. **OutgoingSuccess**: Outgoing payment succeeded, preimage obtained. `SettleIncomingInvoice` action started.
5. **Success**: Incoming invoice settled with preimage. Swap complete.
6. **Failed**: Terminal failure from any non-final state (expiry, permanent error, invalid transition).

## Known TODOs in the Code

1. **Error code categorization** (`error.rs`): All CCH RPC errors currently use `CALL_EXECUTION_FAILED_CODE`. They should be categorized into distinct error codes for better client handling.
2. **Fee setting for LND payments** (`send_outgoing_payment.rs`, `settle_incoming_invoice.rs`): Fee limits are not yet set on LND `SendPaymentRequest` and settle operations.
3. **Hex type for script args** (`config.rs`): `wrapped_btc_type_script_args` should use a proper hex type instead of raw `String`.

## Future Plan

### Short-Term: Robustness and Operational Readiness

1. **Comprehensive error codes**: Categorize CCH errors into distinct JSON-RPC error codes so clients can programmatically distinguish between validation failures, expiry issues, LND connectivity problems, and store errors.

2. **Fee management for LND operations**: Set appropriate fee limits on outgoing LND payments (`SendPaymentRequest.fee_limit_sat` / `fee_limit_msat`) and document the fee model. Without fee limits, LND may choose expensive routes that eat into the hub operator's margin.

3. **Metrics and observability**: Expand the existing `CCH_LND_TRACKER_*` metrics to cover order lifecycle (orders created, succeeded, failed, expired, pruned), action execution counts and latencies, Fiber agent call latencies, and WebSocket reconnection counts.

4. **RPC enhancements**:
   - `list_cch_orders` endpoint with filtering (by status, date range) for operational dashboards.
   - Event subscription (pub/sub) for real-time order status notifications to clients.
   - Pagination support for order queries.

5. **Configuration validation**: Validate the full config at startup (fee parameters, expiry relationships, LND connectivity) and surface clear diagnostics. Currently some misconfigurations are only caught at order creation time.

### Medium-Term: Multi-Asset and Protocol Improvements

6. **Multi-asset cross-chain support**: Currently CCH only supports wrapped BTC (1:1 with Lightning BTC). Extend the protocol and implementation to support other UDT assets on CKB, with configurable exchange rates and asset-specific fee schedules.

7. **PTLC migration path**: As noted in the cross-chain HTLC spec's "Future Works" section, Point Time-Locked Contracts offer improved privacy (payment hash unlinkability across hops), reduced on-chain footprint (adaptor signatures), and better security (wormhole attack resistance). Design the CCH interface to be hash-algorithm-agnostic so the PTLC transition requires minimal changes.

8. **Watchtower integration**: The light paper mentions watchtower services monitoring the cross-chain hub. Implement watchtower support for CCH so that even if the hub operator goes offline, the watchtower can detect and respond to on-chain settlement attempts, protecting both the operator and users.

9. **Order cancellation and refund flow**: Currently failed orders simply record a failure reason. Implement explicit cancellation RPCs and automatic refund mechanisms when the incoming side is locked but the outgoing side fails irrecoverably.

### Long-Term: Scalability and Decentralization

10. **Multi-hub routing**: Allow payments to traverse multiple CCH operators, enabling a decentralized network of cross-chain hubs rather than requiring a single trusted intermediary. This requires cross-hub invoice forwarding and coordinated expiry management.

11. **Hub federation and liquidity management**: Tools for hub operators to manage cross-chain liquidity (rebalancing between CKB and Lightning channels), monitor balance thresholds, and automate channel management.

12. **Support for additional Lightning implementations**: Currently CCH only supports LND via gRPC. Adding support for CLN (Core Lightning) and Eclair would widen the operator base. This could be abstracted behind a `LightningBackend` trait similar to `CchFiberAgentRef`.

13. **Formal verification of the state machine**: The `CchOrderStateMachine` governs fund safety. As the protocol matures, formal verification of the state transition logic and expiry invariants would provide stronger guarantees against fund loss scenarios.

14. **Privacy enhancements**: Implement onion routing for cross-chain payment metadata so that intermediate nodes (in multi-hub scenarios) cannot determine the full payment path or correlate CKB and Lightning identities.
