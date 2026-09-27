//! Modified from [Bevy's CI runner](https://github.com/bevyengine/bevy/tree/main/tools/ci/src)

use xshell::{cmd, Shell};

// A bitflag for each group of checks that CI can run.
const FORMAT: u32 = 1 << 0;
const CLIPPY: u32 = 1 << 1;
const TEST: u32 = 1 << 2;
const DOC_TEST: u32 = 1 << 3;
const DOC_CHECK: u32 = 1 << 4;
const COMPILE_CHECK: u32 = 1 << 5;

const ALL: u32 = FORMAT | CLIPPY | TEST | DOC_TEST | DOC_CHECK | COMPILE_CHECK;

const CLIPPY_FLAGS: [&str; 2] = ["-Aclippy::type_complexity", "-Dwarnings"];

fn main() {
    // When run locally, results may differ from actual CI runs triggered by
    // .github/workflows/ci.yml
    // - Official CI runs latest stable
    // - Local runs use whatever the default Rust is locally

    let arguments = [
        ("lints", FORMAT | CLIPPY),
        ("test", TEST),
        ("doc", DOC_TEST | DOC_CHECK),
        ("compile", COMPILE_CHECK),
        ("format", FORMAT),
        ("clippy", CLIPPY),
        ("doc-test", DOC_TEST),
        ("doc-check", DOC_CHECK),
    ];

    let what_to_run = if let Some(arg) = std::env::args().nth(1) {
        match arguments.iter().find(|(name, _)| *name == arg) {
            Some((_, checks)) => *checks,
            None => {
                let names = arguments
                    .iter()
                    .map(|(name, _)| *name)
                    .collect::<Vec<_>>()
                    .join(", ");
                println!("Invalid argument: {arg:?}.\nEnter one of: {names}.");
                return;
            }
        }
    } else {
        ALL
    };

    let sh = Shell::new().unwrap();

    if what_to_run & FORMAT != 0 {
        // See if any code needs to be formatted
        cmd!(sh, "cargo fmt --all -- --check")
            .run()
            .expect("Please run `cargo fmt --all` to format your code.");
    }

    if what_to_run & CLIPPY != 0 {
        // See if clippy has any complaints.
        cmd!(
            sh,
            "cargo clippy --workspace --all-features -- {CLIPPY_FLAGS...}"
        )
        .run()
        .expect("Please fix `cargo clippy` errors with all features enabled.");

        // Check the examples with clippy
        cmd!(sh, "cargo clippy --examples -- {CLIPPY_FLAGS...}")
            .run()
            .expect("Please fix `cargo clippy` errors in the examples.");
    }

    if what_to_run & TEST != 0 {
        // Run all tests except doc tests, which are handled by the `doc` command.
        cmd!(sh, "cargo test --workspace --lib --bins --tests --benches")
            .run()
            .expect("Please fix failing tests in output above.");
    }

    if what_to_run & DOC_TEST != 0 {
        // Run doc tests
        cmd!(sh, "cargo test --workspace --doc")
            .run()
            .expect("Please fix failing doc-tests in output above.");
    }

    if what_to_run & DOC_CHECK != 0 {
        // Check that building docs work and does not emit warnings
        std::env::set_var("RUSTDOCFLAGS", "-D warnings");
        cmd!(
            sh,
            "cargo doc --workspace --all-features --no-deps --document-private-items"
        )
        .run()
        .expect("Please fix doc warnings in output above.");
    }

    if what_to_run & COMPILE_CHECK != 0 {
        // Check for errors with no features enabled
        cmd!(sh, "cargo check --workspace --no-default-features")
            .run()
            .expect("Please fix `cargo check` errors with no features enabled.");

        // Check for errors with default features enabled
        cmd!(sh, "cargo check --workspace")
            .run()
            .expect("Please fix `cargo check` errors with default features enabled.");
    }
}
