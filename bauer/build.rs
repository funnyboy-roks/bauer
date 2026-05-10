fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let Some((version, _, _)) = version_check::triple() else {
        return;
    };
    let (major, minor, _patch) = version.to_mmp();
    if (major, minor) == (1, 85) {
        println!("cargo:rustc-cfg=using_msrv");
    }
}
