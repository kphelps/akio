extern crate protoc_rust_grpc;

fn main() {
    protoc_rust_grpc::run(protoc_rust_grpc::Args {
        out_dir: "src/rpc",
        includes: &["proto"],
        input: &["proto/api.proto"],
        rust_protobuf: true,
    }).expect("protoc-rust-grpc");
}
