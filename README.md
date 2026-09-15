# grocery-list

A from-scratch Rust rebuild of a previous Ruby CLI grocery list app, built piecemeal as a way to learn Rust. Work in progress.

## Plan

- Start with small standalone Rust exercises to build fundamentals.
- Then port the original app's functionality one vertical slice at a time: catalog (YAML sections/items, validate, format), working list (add/remove/persist), fuzzy search, interactive CLI.
- Functional differences from the original are decided per-feature as each slice is built, not planned upfront.

Local data (catalog, working list) lives under `/data`, which is gitignored, matching the original app's pattern of hand-edited/program-managed files kept out of version control.
