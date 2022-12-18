use std::path::Path;

use nargo_project::Config;

use crate::errors::CliError;

/// Parses a Nargo.toml file from it's path
/// The path to the toml file must be present.
/// Calling this function without this guarantee is an ICE.
pub fn parse<P: AsRef<Path>>(path_to_toml: P) -> Result<Config, CliError> {
    let toml_as_string =
        std::fs::read_to_string(&path_to_toml).expect("ice: path given for toml file is invalid");

    match parse_toml_str(&toml_as_string) {
        Ok(cfg) => Ok(cfg),
        Err(msg) => {
            let path = path_to_toml.as_ref();
            Err(CliError::Generic(format!("{}\n Location: {}", msg, path.display())))
        }
    }
}

fn parse_toml_str(toml_as_string: &str) -> Result<Config, String> {
    match toml::from_str::<Config>(toml_as_string) {
        Ok(cfg) => Ok(cfg),
        Err(err) => {
            let mut message = "input.toml file is badly formed, could not parse\n\n".to_owned();
            // XXX: This error is not always that helpful, but it gives the line number
            // and the entry, in those cases
            // which is useful for telling the user where the error occurred
            // XXX: maybe there is a way to extract ErrorInner from toml
            message.push_str(&err.to_string());
            Err(message)
        }
    }
}
