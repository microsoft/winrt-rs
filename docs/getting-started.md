# Getting Started 

This tutorial will show the basics of how to use the `windows` crate. 

This is *not* meant as a tutorial on Windows programming, and it assumes at least passing familiarity with the Windows API including Win32 and COM. Nor is this meant as an introduction to programming in Rust for Windows developers. You should also have a basic understanding of Rust and Cargo.

The example used here is that of a simple spellchecker utility app. The app takes a string from stdin and returns back various spellchecking suggestions. You can find the complete code for this example [here](../examples/spellchecker).

## Bootstrapping the project

To get started, create a new Rust project named "spellchecker":

```powershell
cargo new spellchecker
```

We'll be generating bindings to the Windows API on the fly from metadata. This means that as we compile our program, the `windows` crate will read in specialized metadata which describes the entire Windows API surface and generates just the code we need to interact with the APIs we want to.

Now we have a choice to make. We _could_ stick the generated code into our `spellchecker` crate directly. However, due to how Rust compiles code, doing so would mean that our bindings are recompiled every time we want to compile our crate. In order to avoid this, we can instead put our bindings into a dedicated `bindings` crate which will be compiled once and cached on future recompilations.

To do this, we'll simply crate a `bindings` crate which our `spellchecker` crate will take a dependency on. Inside of the `spellchecker` crate's directory, run:

```powershell
cargo new bindings --lib
```

We can generate our bindings inside of the `bindings` crate. To do this, we'll need to add the `windows` crate as both a regular dependency and build dependency of the `bindings` crate. In the Cargo.toml for the `bindings` crate add the following:

```toml
# ^^ The rest of the Cargo.toml remains the same
[dependencies]
windows = { git = "https://github.com/microsoft/windows-rs" }  # TODO: change this to the crates.io version when that gets released

[build-dependencies]
windows = { git = "https://github.com/microsoft/windows-rs" }  # TODO: change this to the crates.io version when that gets released
```

Now that the `bindings` crate depends on the `windows` crate, we can generate our bindings inside of a build script. In case you're not familiar with build scripts, they're simply Rust files that get run automatically by cargo when building a Rust crate. To add one, we simply put a "build.rs" file in the `bindings` crate's directory. We'll add the `windows::build!` macro which is where we declare which Windows APIs we want to generate bindings for. To start, we'll declare we want to generate bindings for the `ISpellCheckerFactory` interface as well as the `ISpellChecker` COM class. In the build.rs file add the following:

> TODO: add section describing how we know which namespaces to include. Show the `windows-docs-rs` generated docs.

```rust
fn main() {
    windows::build!(
        // Note that we're using the `intl` namespace which is nested inside the `win32` namespace
        // which itself is inside the `windows` namespace.
        windows::win32::intl::{SpellCheckerFactory, ISpellCheckerFactory},
    )
}
```

We're not done! This generates the bindings, but it puts them into the target folder of our crate. We want to actually use this generated code not just generate the code and forget about it. In order for the generated code to be exported from the `bindings` crate, we need to include it in the bindings crate. Change the lib.rs file of the bindings crate to the following:

```rust
::windows::include_bindings!();
```

This effectively copy/pastes the generated code into the `lib.rs` file. The `bindings` crate now exports all the bindings we've generated. Next, we need to make sure our `spellchecker` crate depends on the `bindings` crate. In the `spellchecker` crate's Cargo.toml file, add the `bindings` crate as a dependency. We'll also add the `windows` crate as a dependency since we'll be using that as well in our `spellchecker` crate.

```toml
# ^^ The rest of the Cargo.toml remains the same
[dependencies]
bindings = { path = "bindings" }
windows = { git = "https://github.com/microsoft/windows-rs" }  # TODO: change this to the crates.io version when that gets released
```

And we're done bootstraping the project. Now we'll move on to the code of the `spellchecker` crate which will use both the generated bindings from the `bindings` crate as well as helper functionality from the `windows` crate.

## Writing the code

Inside of the main.rs file for the `spellchecker` crate, we'll start by adding the set up code. We'll initialize the COM runtime on the main thread. We'll make the `main` function return a `windows::Result` which is simply a standard `Result` type with the error hardcoded as a `windows::Error`.

```rust 
fn main() -> window::Result<()> {
    // initialize the main thread as a multithreaded apartment
    windows::initialize_mta()?;
    // The rest of the code will go here!
    Ok(())
}
```

Next, we'll initialize the `ISpellCheckerFactory` which is what gives us access to spellcheckers. We'll first make sure the `intl` namespace is in scope at the top of the main.rs file:

```rust
use bindings::windows::win32::intl;
```

Then we can do the initialization:


```rust

fn main() -> window::Result<()> {
    // initialize the main thread as a multithreaded apartment
    windows::initialize_mta()?;

    // Create ISpellCheckerFactory
    let factory: intl::ISpellCheckerFactory = windows::create_instance(&intl::SpellCheckerFactory)?;

    // The rest of the code will go here!
    Ok(())
}
```

> TODO: Finish