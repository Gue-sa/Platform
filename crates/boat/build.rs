fn main() {
    let config: slint_build::CompilerConfiguration =
        slint_build::CompilerConfiguration::default().with_style("fluent-light".into());

    slint_build::compile_with_config("./src/ui/appwindow.slint", config).unwrap();
}
