fn main() {
    let config = slint_build::CompilerConfiguration::new().with_library_paths(slint_access_widgets::library_paths());
    slint_build::compile_with_config("ui/app.slint", config).unwrap();
}
