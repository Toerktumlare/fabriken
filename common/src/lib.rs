pub mod channels;
pub mod executor;
pub mod models;
pub mod parser;
pub mod runtime;
pub mod scheduler;

pub mod build {
    use tonic::include_proto;

    include_proto!("build");
}

pub mod startup {
    use tonic::include_proto;

    include_proto!("startup");
}
