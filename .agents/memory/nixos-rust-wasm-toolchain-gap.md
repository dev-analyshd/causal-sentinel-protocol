---
name: NixOS sandbox cannot build Rust to wasm32-unknown-unknown
description: Explains why Rust contract/wasm compilation fails in this Replit sandbox and what was tried.
---

The Nix-provided Rust toolchain (`rust-stable` module) has no `wasm32-unknown-unknown`
std lib installed, and there is no Nix module that adds it.

Workaround attempted: installed `rustup` via `installSystemDependencies`, then
`rustup toolchain install stable` + `rustup target add wasm32-unknown-unknown`.
The toolchain downloads fine, but the prebuilt `rustc`/`rustc_driver` binaries
crash with `cannot allocate memory in static TLS block` — a known NixOS
incompatibility with non-Nix-built ELF binaries that have large thread-local
storage. `patchelf --set-interpreter`/`--set-rpath` to the Nix glibc did not
fix it (dependent .so files still fail to load correctly).

**Why:** NixOS's non-FHS layout and TLS allocation model breaks generic
prebuilt Linux binaries; this is a class of problem, not specific to Rust.

**How to apply:** Don't re-attempt the rustup+patchelf path for wasm32 (or
any other target needing a prebuilt toolchain) unless a nix-ld-based fix or
a nixpkgs-native rust-with-wasm-target package becomes available. For any
Casper/Odra (or other wasm-target) contract work, plan for compilation to
happen outside this sandbox (user's machine, CI, or Casper's own dev
containers) — fix source/Cargo.toml bugs here, but don't promise a working
`.wasm` artifact from this environment.

Two further routes were tried and also confirmed blocked (2026-07-05):
1. `nix build`/`nix eval` against the `fenix` flake (community rust-with-
   extra-targets overlay): the flake itself fetches fine, but evaluating
   ANY of its toolchain outputs (even a pure attribute read, no download)
   hangs indefinitely and must be killed — some network path fenix needs
   at eval time is blocked/unreachable in this sandbox, distinct from the
   plain `https://static.rust-lang.org` host which IS reachable via curl.
2. Stock nixpkgs has no `wasm32-unknown-unknown` `pkgsCross` target, and
   `nixpkgs#rustc.targetPlatforms` (verified via fast successful `nix
   eval`, so this is a real fact not a network artifact) lists
   `wasm32-wasi` but never `wasm32-unknown-unknown` — the precompiled
   nixpkgs rustc physically cannot emit the target Odra/most wasm
   contract tooling needs, independent of any network issue.

Conclusion: three structurally different routes (rustup binaries, flake
overlay, nixpkgs cross target) fail for three different root causes. This
is a firm platform limitation, not something worth re-attempting without a
fundamentally new approach (e.g. a from-scratch rustc source build, which
would take hours and is likely still infeasible in this sandbox).
