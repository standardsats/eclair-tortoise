## Building

### Dependencies

Compiling from sources requires OpenSSL libraries. On Linux distros try to install `openssl-devel` in package managers.

### Compilation

`rust` and `cargo` need to be installed.

Use 

```
cargo check
```
to ensure that you have correct environment.

If it shows error. Try

```
rustup update nightly
```
It will update rust and cargo as well. Then repeat `check` and if 
it finishes, proceed with `cargo build`.  
