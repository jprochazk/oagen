### TypeScript API Type Generator

Command line tool that transforms OpenAPI (Swagger) or API Blueprint (Apiary) specifications into TypeScript.

Development is in progress.

## Installation

1. Install [Rust](https://www.rust-lang.org/)
2. Run `cargo install --git https://github.com/jprochazk/api-type-gen.git`
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

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
