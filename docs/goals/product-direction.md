# Product Direction

> Status: active baseline

## Positioning

`rustzen-admin` is an open-source Rust full-stack admin foundation.

It is not a vertical product and it is not a template dump. The repository exists to provide a clear, maintainable base for real admin systems with explicit backend, frontend, permission, and deployment boundaries.

## Target Outcome

- keep the monorepo small and understandable
- keep auth and permission reusable
- keep backend features self-contained
- keep frontend and backend contracts synchronized
- keep documentation usable as an engineering interface

## Non-Goals

- do not turn the repository into a multi-runtime platform unless the product scope requires it
- do not add compatibility layers or fallback paths to preserve old structure
- do not expand documentation into a heavy process system

## Repository Direction

- strengthen the repository as a clean admin foundation
- improve deployment and packaging clarity when needed
- grow feature depth without losing structural clarity
- keep documentation governance explicit as the codebase evolves
