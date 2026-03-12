---
name: run-e2e-test
description: Run an e2e test for the Fiber Network Node project
---
# Run E2E Test

Run a Bruno CLI e2e test suite against local Fiber Network Node instances.

The user provides a test suite name as an argument, e.g. `/run-e2e-test cross-chain-hub-separate`. Use that name as `$TEST_SUITE` throughout. If no name is provided, ask the user to pick one from the list below.

## Available test suites

Located under `tests/bruno/e2e/`:

- `3-nodes-transfer`
- `cross-chain-hub` (requires bitcoind + lnd)
- `cross-chain-hub-separate` (requires bitcoind + lnd)
- `funding-tx-verification`
- `invoice-ops`
- `open-use-close-a-channel`
- `period-check`
- `reestablish`
- `router-pay`
- `shutdown-force`
- `udt`
- `udt-router-pay`
- `watchtower/*` (multiple sub-suites, e.g. `watchtower/force-close`)

## Procedure

### 1. Build

Build the main binary and the udt-init config generator:

```bash
cargo build --locked
cd tests/deploy/udt-init && cargo build --locked && cd -
```

Wait several minutes for cargo builds to complete.

### 2. Clean state (if needed)

If tests failed previously or you need a fresh start, kill all processes and set environment variable `REMOVE_OLD_STATE=y` when running `start.sh`:

```bash
# Kill existing processes
pkill -9 -f 'fnn' 2>/dev/null
pkill -9 -f 'ckb run' 2>/dev/null
pkill -9 -f 'lnd ' 2>/dev/null
pkill -9 -f 'bitcoind' 2>/dev/null
sleep 2
```

### 3. Start nodes

Run `start.sh` in the **background** from the repo root. It blocks forever (monitoring child processes), so it must not be run in the foreground of a tool call with a timeout.

```bash
./tests/nodes/start.sh "e2e/$TEST_SUITE" &
```

What `start.sh` does:

1. For cross-chain-hub tests: runs `tests/deploy/lnd-init/setup-lnd.sh` (starts bitcoind, creates wallets, starts lnd-bob and lnd-ingrid, opens a lightning channel)
2. Runs `tests/deploy/init-dev-chain.sh` which, if `tests/deploy/node-data/` doesn't exist:
   - Initializes a CKB dev chain
   - Starts CKB temporarily to transfer CKB to node wallets
   - Runs `tests/deploy/deploy.sh` which calls `udt-init` to deploy contracts, create UDT accounts, and generate `config.yml` for each node
   - Kills the temporary CKB
3. Starts CKB node (port 8114)
4. Starts FNN nodes 1 (port 21714), 2 (port 21715), 3 (port 21716) from `tests/nodes/` directory
5. For `cross-chain-hub-separate`: also starts standalone CCH node (port 21717)

### 4. Wait for readiness

Wait for all required ports to be listening before running tests:

```bash
# Standard tests need: 8114 (CKB), 21714-21716 (FNN nodes)
# Cross-chain tests also need: 18443 (bitcoind), 10009/11009 (lnd), 21717 (CCH)

# Poll until ready (timeout after ~2 minutes)
for i in $(seq 1 24); do
  if ss -tlnp | grep -q '21714' && ss -tlnp | grep -q '21716'; then
    echo "Nodes ready"; break
  fi
  sleep 5
done
```

Alternatively, use `./tests/nodes/wait.sh` which reads ports from `tests/nodes/.ports` (only created during `init-dev-chain.sh` first run). If `.ports` doesn't exist, poll the known ports directly.

### 5. Run the Bruno tests

```bash
cd tests/bruno
npm exec -- @usebruno/cli@1.20.0 run "e2e/$TEST_SUITE" -r --env test
```

Optional arguments:

- `--env xudt-test` for xUDT variant of the udt test
- `--env-var HASH_ALGORITHM=sha256` for sha256 variant of 3-nodes-transfer
- `--env-var FUNDING_TX_VERIFICATION_CASE=<case>` for funding-tx-verification (`remove_change`, `modify_change`, `fund_from_peer`)

### 6. Interpret results

Bruno outputs:

- `Requests: N passed, M total`
- `Assertions: N passed, M total`
- Individual request results with assertion pass/fail details

On failure, check:

- FNN node logs: `tests/nodes/node*.log` or `tests/nodes/*.log`
- CKB logs: look at CKB process stderr
- LND logs: `tests/deploy/lnd-init/lnd-bob/logs/` and `tests/deploy/lnd-init/lnd-ingrid/logs/`

Common failure patterns:

- **"can not find enough UDT owner cells"**: Node doesn't have UDT cells. Needs a clean restart (remove `node-data/` and re-initialize).
- **Port already in use**: Previous processes not fully killed. Force kill and verify ports are free.
- **Assertion mismatch on HTTP status**: Check the pre-script in the `.bru` file; some tests use pre-scripts that make HTTP calls and set variables.

## Key files

| File | Purpose |
|------|---------|
| `tests/nodes/start.sh` | Orchestrates node startup |
| `tests/nodes/wait.sh` | Waits for ports to be ready |
| `tests/deploy/init-dev-chain.sh` | Initializes CKB dev chain and wallets |
| `tests/deploy/deploy.sh` | Deploys contracts via udt-init |
| `tests/deploy/udt-init/src/main.rs` | Deploys UDT contracts, creates accounts, generates node configs |
| `tests/deploy/lnd-init/setup-lnd.sh` | Sets up bitcoind + LND for cross-chain tests |
| `tests/bruno/environments/test.bru` | Environment variables (ports, pubkeys, URLs) |
| `tests/nodes/deployer/config.yml` | Template config for node config generation |
| `tests/nodes/<N>/config.yml` | Generated per-node config (not checked in) |
| `.github/workflows/e2e.yml` | CI workflow definition |

## Environment variables

| Variable | Purpose |
|----------|---------|
| `REMOVE_OLD_STATE=y` | Force re-initialization of all state |
| `REMOVE_OLD_FIBER=y` | Only clean fiber store, keep chain state |
| `CCH_SEPARATE=y` | Start standalone CCH node (set automatically for cross-chain-hub-separate) |
| `START_BOOTNODE=y` | Start bootnode (set automatically for router-pay) |
| `TEST_ENV=debug\|release` | Build profile, defaults to debug |
