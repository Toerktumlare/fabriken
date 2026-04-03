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
            "src/proto/status/health_service.proto",
            "src/proto/status/health_types.proto",
        ],
        &["src/proto/build", "src/proto/status"],
    )?;
    Ok(())
}
