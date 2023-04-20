use std::io::Result;

fn main() -> Result<()> {
    let protos = &["src/pb/ipns_entry.proto"];
    let includes = &["src/"];
    prost_build::compile_protos(protos, includes)?;
    Ok(())
}
