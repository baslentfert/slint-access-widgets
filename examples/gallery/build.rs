fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .with_library_paths(slint_access_widgets::library_paths())
        // The .po files made by `po-bundle update` (the widgets' own texts
        // come in from the crate, already translated).
        .with_bundled_translations("translations");
    slint_build::compile_with_config("ui/app.slint", config).unwrap();
}
