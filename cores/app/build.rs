fn main() {
    for name in ["SKIM_SEARCH_BUILD_VERSION", "SKIM_SEARCH_BUILD_SHA"] {
        println!("cargo:rerun-if-env-changed={name}");
        let default = if name.ends_with("VERSION") { env!("CARGO_PKG_VERSION") } else { "unassigned" };
        let value = std::env::var(name).unwrap_or_else(|_| default.to_string());
        println!("cargo:rustc-env={name}={value}");
    }
    for name in ["SKIM_SEARCH_BUILD_VERSION", "SKIM_SEARCH_BUILD_SHA"] {
        println!("cargo:rerun-if-env-changed={name}");
        let fallback = if name.ends_with("VERSION") { env!("CARGO_PKG_VERSION") } else { "unassigned" };
        let value = std::env::var(name).unwrap_or_else(|_| fallback.to_string());
        println!("cargo:rustc-env={name}={value}");
    }
    // Dark theme (handover §24).
    // Debug info lets cores/tests find elements by accessible-label (i-slint-backend-testing).
    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent-dark".into())
        .with_debug_info(true);
    slint_build::compile_with_config("ui/main.slint", config).expect("compile ui/main.slint");
}
