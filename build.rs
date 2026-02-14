use std::io::Result;
fn main() -> Result<()> {
    //protos/google/ai/generativelanguage/v1
    //https://github.com/googleapis/googleapis/archive/refs/heads/master.zip
    /*tonic_prost_build::configure()
    .build_server(false) //not needed
    .compile_well_known_types(true) //those are definitely needed
    .compile_protos(
        &[
            "./protos/google/ai/generativelanguage/v1/generative_service.proto",
            "./protos/google/cloud/aiplatform/v1/prediction_service.proto",
        ],
        &["protos"],
    )?;*/
    /*tonic_prost_build::compile_protos(
        ,
    )?;*/
    Ok(())
}
