use std::path::PathBuf;

// TODO: Change to eyre, then carry useful information like "while deserializing toml"
pub fn upgrade_std(filename: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let output_filename = filename.with_extension("yml");

    if output_filename.exists() {
        // TODO: This should be an Err
        eprintln!(
            "{} already exists, delete it to prevent overriding",
            output_filename.display()
        );

        return Ok(());
    }

    let v1_std: selene_lib::standard_library::v1::StandardLibrary =
        toml::from_str(&std::fs::read_to_string(&filename)?)?;

    let modern_std: selene_lib::standard_library::StandardLibrary = v1_std.into();

    std::fs::write(&output_filename, serde_yaml::to_string(&modern_std)?)?;

    Ok(())
}
