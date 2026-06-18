# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅        |

## Reporting vulnerabilities

If you discover a security issue, please report it responsibly:

1. **Do not** open a public GitHub issue for security vulnerabilities.
2. Contact the maintainers directly or use Polymarket's official security channel if contributing upstream.

## SDK security properties

This SDK is designed for safe integration in production:

- **Transport**: All API calls use HTTPS with `rustls` (no native TLS / OpenSSL dependency).
- **Timeouts**: Default 30-second request timeout prevents hung connections.
- **Secrets**: Never log or persist private keys. The `secure` feature reads credentials from environment (`POLYMARKET_PRIVATE_KEY`) or caller-supplied config only.
- **Input validation**: IDs, addresses, URLs, and pagination params are validated before requests are sent.
- **Error sanitization**: HTML error pages from upstream proxies are not returned verbatim to callers.

## Dependency hygiene

Run periodically:

```bash
cargo audit
```

## Safe usage

- Store API keys and private keys in environment variables or a secrets manager — never in source code.
- Pin SDK versions in production `Cargo.lock`.
- Use `Environment::production()` unless explicitly testing against staging.
