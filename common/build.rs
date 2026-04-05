use std::io::Result;

fn main() -> Result<()> {
    let protoc = protoc_bin_vendored::protoc_bin_path().unwrap();

    unsafe {
        std::env::set_var("PROTOC", protoc);
    }

    tonic_prost_build::configure().compile_protos(
        &[
            "src/proto/build/build_service.proto",
            "src/proto/build/build_types.proto",
            "src/proto/startup/startup_service.proto",
            "src/proto/startup/startup_types.proto",
        ],
        &["src/proto"],
    )?;
    Ok(())
}
