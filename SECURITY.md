# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.0.0-draft | Yes |

## Reporting a Vulnerability

Do not open a public issue for security vulnerabilities.

Send a detailed report to the project security contact. Include:

- Affected component(s): grammar parser, kernel sandbox, MCP endpoint, CI/CD
- Attack vector description
- Proof-of-concept exploit
- Suggested mitigation

The security team will acknowledge receipt within 48 hours and provide a
timeline for remediation. Critical vulnerabilities will trigger an out-of-band
security release.

## Scope

Security reports are accepted for:

- Kernel sandbox escapes (VirtualHeap, I/O bypass)
- Parser-level crashes or undefined behavior on malicious `.ueas` input
- AST injection via crafted canonical JSON
- MCP endpoint authentication bypass
- CI/CD supply chain compromise

Out of scope:

- Theoretical attacks requiring physical access
- Social engineering
- Unsafe usage of transpiled output in production systems (the standard
  provides verification, not runtime sandboxing of transpiled code)
