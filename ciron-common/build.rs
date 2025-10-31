fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/generated")
        .compile_protos(
            &["../proto/ciron.proto", "../proto/errors.proto"],
            &["../proto"],
        )?;
        
    Ok(())
}
