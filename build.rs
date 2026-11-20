fn main() {
    for path in [
        "windows/app.rc",
        "windows/phone-region.ico",
        "windows/app.manifest",
        "windows/third-party-notices.zip",
    ] {
        println!("cargo:rerun-if-changed={path}");
    }
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        // Supply our manifest and version metadata to both GNU and MSVC links.
        if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
            println!("cargo:rustc-link-arg-bin=PhoneRegion=/MANIFEST:NO");
        }
        embed_resource::compile_for("windows/app.rc", ["PhoneRegion"], embed_resource::NONE)
            .manifest_required()
            .expect("could not compile Windows resources");
    }
}
