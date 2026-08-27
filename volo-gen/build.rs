fn main() {
    volo_build::Builder::thrift()
        .add_service("../idl/user.thrift")
        .write()
        .expect("Failed to generate Thrift code");
}
