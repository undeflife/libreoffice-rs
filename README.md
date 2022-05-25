# libreofficeKit-rs

Rust bindings to [LibreOfficeKit](https://docs.libreoffice.org/libreofficekit.html)


## Installation

```toml
[dependencies]
libreoffice-rs = 0.1
```
you need install LibreOffice, Debian 11 for example: 
```bash
$ sudo apt-get install libreoffice-core libreofficekit-dev
```
set env variable `LO_INCLUDE_PATH` to the LibreOffice headers.

due to [this issue](https://github.com/rust-lang/rust-bindgen/issues/1090) , here use a libwrapper.a to carry `static funtion lok_init` which defined in `LibreOfficeKitInit.h`, so you may also need gcc installed.

```c

## Example

```rust
  use libreoffice_rs::Office;
  // your libreoffice installation path
  let mut office = Office::new("/usr/lib/libreoffice/program");
  let mut doc = office.document_load("/tmp/test.doc");
  doc.save_as("/tmp/test.pdf", "pdf", None);
```
