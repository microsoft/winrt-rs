# Windows SpellChecker

An example repository of how to use the [`windows`](https://crates.io/crates/windows) crate. This app uses various Windows APIs for checking text from standard in and displaying corrections in standard out.

## Running

Running the app is easy. Make sure you have [Rust installed](https://rustup.rs), and then run the following:

```powershell
cargo run -- "This is text with a word missspelled!
```