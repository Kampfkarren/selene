use std::path::Path;

pub fn upgrade_std(filename: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let output_filename = filename.with_extension("yml");

    if output_filename.exists() {
        return Err(format!(
            "{} already exists, delete it before re-running",
            output_filename.display()
        )
        .into());
    }

    let v1_std: selene_lib::standard_library::v1::StandardLibrary =
        toml::from_str(&std::fs::read_to_string(filename)?)?;

    let modern_std: selene_lib::standard_library::StandardLibrary = v1_std.into();

    std::fs::write(&output_filename, serde_yaml::to_string(&modern_std)?)?;

    Ok(())
}
