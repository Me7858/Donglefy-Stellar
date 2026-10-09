# Donglefy-Stellar

A Soroban smart contract that splits token payments among recipients by fixed shares.

## How it works

1. `init(token, recipients, shares)`: set the payout token, the recipients and their
   share weights. Callable once; every share must be greater than zero.
2. Anyone sends tokens to the contract address.
3. `distribute()`: pays out the full balance proportionally. Rounding dust goes to the
   last recipient, so the contract balance always ends at zero. Anyone may call it.

Read-only: `recipients()`, `shares()`.

## Errors

| Code | Name | Meaning |
|------|------|---------|
| 1 | AlreadyInitialized | `init` was called twice |
| 2 | NotInitialized | contract not configured yet |
| 3 | InvalidShares | empty list, length mismatch, zero or overflowing shares |
| 4 | NothingToDistribute | contract holds no tokens |

## Develop

```bash
cargo test --workspace
cargo build --release --target wasm32-unknown-unknown
```

Deploy `init` in the same transaction flow as deployment so nobody else can configure
the contract first.

## License

MIT
