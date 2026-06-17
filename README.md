# STEF

STEF is a compact, deterministic binary serialization format with typed, self-describing values.

This repository contains the official Rust implementation.

## Crates

| Crate                           | Description                                 |
|---------------------------------|---------------------------------------------|
| [`stef-core`](crates/stef-core) | Raw serialization and deserialization layer |

## Roadmap

**Phase 1 — Foundation**
- [x] `stef-core` — complete rewrite, stable
- [ ] `stef-fuzz` — fuzz testing layer over `stef-core`

**Phase 2 — High-level API**
- [ ] `stef-derive` — derive macros for `StefSerialize` and `StefDeserialize`
- [ ] `stef` — traits and high-level API over `stef-core`
- [ ] `stef-json` — bidirectional JSON ↔ STEF bridge

**Phase 3 — Interoperability**
- [ ] `stef-ffi` — C ABI layer
- [ ] `stef-json-ffi` — FFI for the JSON bridge
- [ ] Under consideration: `stef-yaml`, `stef-toml`, `stef-bson`

**Phase 4 — Tooling**
- [ ] `stef-cli` — VIM-style TUI (ratatui) for inspecting and navigating STEF files
- [ ] Plugin system as dynamic libraries via C ABI (depends on `stef-ffi`)
- [ ] VSCode extension

**Phase 5 — Schemas**
- [ ] `stef-schema` — schema layer over `stef-derive`: runtime validation and interoperable description

**Ongoing**
- [ ] Bilingual documentation (spec, ADRs, READMEs)
- [ ] `docs` branch with Starlight/Astro

## License

MIT
