# Publishing to crates.io

Publish in dependency order. You need a [crates.io API token](https://crates.io/settings/tokens).

```bash
cargo login   # paste token once

# 1. types
cargo publish -p polymarket-types

# 2. bindings (wait ~1 min for index)
cargo publish -p polymarket-bindings

# 3. client
cargo publish -p polymarket-client
```

Verify before publishing:

```bash
cargo publish -p polymarket-types --dry-run --allow-dirty
cargo test
cargo clippy -p polymarket-client --all-targets --features secure -- -D warnings
```

After publish:

- https://crates.io/crates/polymarket-client
- https://docs.rs/polymarket-client (builds automatically)

## Crate names

| Crate | crates.io |
|-------|-----------|
| `polymarket-types` | primitives |
| `polymarket-bindings` | API models |
| `polymarket-client` | main SDK |
