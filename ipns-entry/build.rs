// In a Cargo worspace, the `build.rs` file goes in the root of the workspace.
// This means the build dependencies are shared across all crates in the workspace.

use std::io::Result;

fn main() -> Result<()> {
    // protos - Paths to .proto files to compile. Any transitively imported .proto files are automatically be included.
    let protos = &["src/pb/ipns_entry.proto"];

    // includes - Paths to directories in which to search for imports.
    // Directories are searched in order.
    // The .proto files passed in protos must be found in one of the provided include directories.
    let includes = &["src/"];

    // https://docs.rs/prost-build/latest/prost_build/fn.compile_protos.html#arguments
    prost_build::compile_protos(protos, includes)?;
    Ok(())
}
