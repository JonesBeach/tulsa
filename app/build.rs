extern crate prost_build;

fn main() {
    prost_build::compile_protos(&["src/proto/gtfs-realtime.proto"], &["src/"]).unwrap();
}
