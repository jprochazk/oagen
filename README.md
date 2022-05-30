### TypeScript API Type Generator

Command line tool that transforms OpenAPI (Swagger) or API Blueprint (Apiary) specifications into TypeScript.

Development is in progress.

## Installation

1. Install [Rust](https://www.rust-lang.org/)
2. Run `cargo install --git https://github.com/keboola/api-type-gen.git`
3. CLI is available globally as `tsapi`

**Commands**
- `help`
- `oapi <input> <output>`
- 🚧 `apib <input> <output>`

Commands marked with 🚧 are not yet implemented.

Examples:
```
$ tsapi oapi my-api.json my-api.ts
```

## Usage

1. Install [Rust](https://www.rust-lang.org/)
2. Run `cargo run -- <args>` where `<args>` are the same as described above in `Installation`.

## Running tests

1. Install [Rust](https://www.rust-lang.org/)
2. Run `cargo test`
## License

MIT licensed, see [LICENSE](./LICENSE) file.
