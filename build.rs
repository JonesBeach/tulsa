extern crate prost_build;

// Helpful example from: https://github.com/danburkert/snazzy
fn main() {
    prost_build::compile_protos(&["src/gtfs-realtime.proto"], &["src/"]).unwrap();
}
