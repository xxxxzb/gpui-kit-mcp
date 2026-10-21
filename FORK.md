# gpui-kit-mcp fork

This repository is the `gpui-kit-mcp` fork of
[`themixednuts/gpui-mcp`](https://github.com/themixednuts/gpui-mcp). It keeps
the upstream bridge, MCP server and tool surface, and adds support for the GPUI
Kit release used by this project.

## Base

- Upstream commit: `eb248cc` (2026-10-09)
- Development branch: `gpui-kit`
- Upstream remote: `upstream`

## Divergence

- `gpui-pre` 0.3.9 is vendored and supported, alongside 0.3.5, 0.3.6 and 0.3.7.
- `gpui-base` resolves to 0.7.3.
- The GPUI Kit demo and consumer recipe target `gpui-kit =0.7.3`.
- The 0.3.9 automation patch is a port of the 0.3.7 patch: `src/element.rs` and
  `src/window.rs` needed manual hunks for the newer frame-observer and drawing
  code. `font-fallback.patch` and `grid.patch` apply unchanged.

## Regenerate the vendored tree

```console
cargo xtask vendor --check
cargo xtask vendor
```

`vendor/gpui-pre.json` records the crates.io checksum and Zed revision for each
supported release. `vendor/patches/gpui-pre/0.3.9/` holds the patch series.

## Sync with upstream

```console
git fetch upstream
git rebase upstream/main
```

Expect conflicts in `Cargo.toml`, `vendor/gpui-pre.json`, `vendor/README.md`,
`README.md`, `xtask/src/consumer.rs` and the 0.3.9 patch series. Re-run
`cargo xtask vendor --check`, `cargo test --workspace` and the GPUI Kit demo
tests after resolving them. The 0.3.9 patch series is the part most likely to
need manual work; keep the diff against the 0.3.7 series small so it stays
reviewable.

## License

Apache-2.0, inherited from upstream. Modified files retain the upstream
copyright and license headers.
