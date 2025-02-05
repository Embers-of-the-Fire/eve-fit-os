fn main() -> std::io::Result<()> {
    #[cfg(feature = "protobuf")]
    {
        build_protobuf()?;
    }

    Ok(())
}

#[cfg(feature = "protobuf")]
fn build_protobuf() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=efos.proto");
    prost_build::Config::new().compile_protos(&["efos.proto"], &["."])?;

    Ok(())
}
