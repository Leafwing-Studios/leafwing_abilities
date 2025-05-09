use xshell::{cmd, Shell};

fn main() {
    // When run locally, results may differ from actual CI runs triggered by
    // .github/workflows/ci.yml
    // - Official CI runs latest stable
    // - Local runs use whatever the default Rust is locally

    let sh = Shell::new().unwrap();

    // See if any code needs to be formatted
    cmd!(sh, "cargo fmt --all -- --check")
        .run()
        .expect("Please run `cargo fmt --all` to format your code.");

    // See if clippy has any complaints.
    // - Type complexity must be ignored because we use huge templates for queries
    cmd!(
        sh,
        "cargo clippy --workspace --all-features -- -D warnings -A clippy::type_complexity"
    )
    .run()
    .expect("Please fix `cargo clippy` errors with all features enabled.");

    // Check for errors with no features enabled
    cmd!(sh, "cargo check --workspace --no-default-features")
        .run()
        .expect("Please fix `cargo check` errors with no features enabled .");

    // Check for errors with default features enabled
    cmd!(sh, "cargo check --workspace")
        .run()
        .expect("Please fix `cargo check` errors with default features enabled.");

    // Check the examples with clippy
    cmd!(
        sh,
        "cargo clippy --examples -- -D warnings -A clippy::type_complexity"
    )
    .run()
    .expect("Please fix `cargo clippy` errors for the examples.");
}
