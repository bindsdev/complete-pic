# complete-pic

![](https://img.shields.io/github/actions/workflow/status/bindsdev/complete-pic/ci.yml?style=flat-square)
![](https://img.shields.io/crates/v/complete-pic?style=flat-square)
![](https://img.shields.io/crates/d/complete-pic?style=flat-square)
![](https://img.shields.io/crates/l/complete-pic?style=flat-square)
![](https://img.shields.io/docsrs/complete-pic?style=flat-square)

A complete interface for both the legacy 8259 PIC and the newer APIC. More specific documentation can be found by reading the documentation of the
modules designated for the 8259 PIC and APIC. 

## Usage
To use this crate, add it to your `Cargo.toml` file:
```toml
[dependencies]
complete_pic = "0.2.0"
```
   
### Crate Features
- `8259pic` - Enable interface for the legacy 8259 PIC (not on by default)
- `apic` - Enable interface for the newer APIC (enabled by default; crate assumes you want to use the more modern variant)

## Changelog

View the [changelog](/CHANGELOG.md) to see what changed across crate versions.
