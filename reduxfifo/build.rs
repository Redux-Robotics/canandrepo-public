fn main() {
    let target = std::env::var("TARGET").unwrap();
    if target.contains("linux") {
        // SONAME is necessary for wpilib to function
        println!("cargo:rustc-link-arg=-Wl,-soname,libreduxfifo.so");
    }

    build_data::set_GIT_COMMIT_SHORT().unwrap();
    build_data::set_GIT_DIRTY().unwrap();
    build_data::set_RUSTC_VERSION().unwrap();
}
