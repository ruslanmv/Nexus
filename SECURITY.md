# Security Policy

## Supported Versions

Only the latest released minor version is supported.

## Reporting a Vulnerability

Please report security issues privately (do **not** open a public issue).

Include:
- affected version(s)
- reproduction steps or proof-of-concept
- impact assessment (DoS, data exposure, etc.)

## Hardening Guidelines

If Nexus is used in environments where **untrusted** agents or external message sources exist:

- Enforce authentication/authorization at the boundary (API gateway, IPC auth, etc.).
- Apply message size limits and validation before constructing `Message`.
- Prefer process isolation or sandboxing for untrusted agent code.
