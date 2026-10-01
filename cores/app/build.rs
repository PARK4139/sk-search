fn main() {
    // Dark theme (handover §24).
    // Debug info lets cores/tests find elements by accessible-label (i-slint-backend-testing).
    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent-dark".into())
        .with_debug_info(true);
    slint_build::compile_with_config("ui/main.slint", config).expect("compile ui/main.slint");
}
