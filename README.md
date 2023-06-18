[![master CI badge](https://img.shields.io/github/actions/workflow/status/Alorel/argley-rs/ci.yml?label=master%20CI)](https://github.com/Alorel/argley-rs/actions/workflows/ci.yml?query=branch%3Amaster)
[![crates.io badge](https://img.shields.io/crates/v/argley)](https://crates.io/crates/argley)
[![docs.rs badge](https://img.shields.io/docsrs/argley?label=docs.rs)](https://docs.rs/argley)
[![dependencies badge](https://img.shields.io/librariesio/release/cargo/argley)](https://libraries.io/cargo/argley)

Turn a struct into arguments for a [Command](https://doc.rust-lang.org/stable/std/process/struct.Command.html).

```rust
use argley::prelude::*;

#[derive(Args)]
struct Args<'a> {
  compress: bool,
  
  #[arg(rename = "dir")]
  output_dir: &'a Path,
  
  #[arg(variadic)]
  input_files: Vec<&'a Path>,
}

let args = Args {
  compress: true,
  output_dir: Path::new("./output"),
  input_files: vec![Path::new("./foo.txt"), Path::new("./bar.txt")],
};

let output = Command::new("some-application")
  .add_arg_set(&args)
  .output()
  .unwrap();
```

Support for [async-std](https://crates.io/crates/async-std) and [tokio](https://crates.io/crates/tokio) can be enabled
via their respective features.

See crate-level docs for detailed configuration options.
