extern crate protoc_rust;

fn main() {
    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/rpc",
        includes: &["proto"],
        input: &["proto/api.proto"],
    }).expect("protoc-rust");
}
