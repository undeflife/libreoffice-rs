# libreofficeKit-rs

Rust bindings to [LibreOfficeKit](https://docs.libreoffice.org/libreofficekit.html)


## Installation

```toml
[dependencies]
libreoffice-rs = 0.3.1
```

you need install LibreOffice ( >= 6.0 is recommended ), Debian 11 for example: 
```bash
$ sudo apt-get install libreoffice libreofficekit-dev clang
# set env variable `LO_INCLUDE_PATH` to the LibreOffice headers.
$ export LO_INCLUDE_PATH=/usr/include/LibreOfficeKit
```

due to [this issue](https://github.com/rust-lang/rust-bindgen/issues/1090) , here use a libwrapper.a to carry `static funtion lok_init` which defined in `LibreOfficeKitInit.h`.

## Example

```rust
use libreoffice_rs::{Office, LibreOfficeKitOptionalFeatures, urls};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // your libreoffice installation path
  let mut office = Office::new("/usr/lib/libreoffice/program")?;
  let doc_url = urls::local_into_abs("./test_data/test.odt")?;
  let mut doc = office.document_load(doc_url)?;
  doc.save_as("/tmp/test.pdf", "pdf", None);
  Ok(())
}
```
## License
This project is licensed under the [Apache License 2.0][license]

