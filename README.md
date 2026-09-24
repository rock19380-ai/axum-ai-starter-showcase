# Axum AI Starter — Engineering Showcase

[![CI](https://github.com/rock19380-ai/axum-ai-starter-showcase/actions/workflows/ci.yml/badge.svg)](https://github.com/rock19380-ai/axum-ai-starter-showcase/actions/workflows/ci.yml)

A curated public presentation of production-oriented Rust backend patterns used in a private/commercial AI application starter.

## Engineering areas

- Tokio + Axum request handling
- streaming/SSE boundaries
- typed public errors
- bounded provider timeouts
- request IDs
- cancellation-aware async architecture
- PostgreSQL + SQLx patterns in the private implementation
- health/readiness behavior
- Docker-based local development
- tests and CI

## Runnable public example

`examples/streaming-api` is an independently written demonstration of the public engineering patterns without exposing commercial source.

```bash
cargo test --manifest-path examples/streaming-api/Cargo.toml
```

It demonstrates:

- Axum routing;
- request-ID propagation;
- typed error responses;
- bounded upstream timeouts;
- SSE streaming;
- integration-style endpoint tests.

The commercial/private source remains separate. This repository is intentionally a showcase rather than a mirror of the paid implementation.
