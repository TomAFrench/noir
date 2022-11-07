use hex::FromHexError;
use noirc_abi::errors::InputParserError;
use std::{fmt::Display, io::Write, path::PathBuf};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Debug)]
pub enum CliError {
    Generic(String),
    DestinationAlreadyExists(PathBuf),
    PathNotValid(PathBuf),
    ProofNotValid(FromHexError),
}

impl CliError {
    pub(crate) fn write(&self) -> ! {
        let mut stderr = StandardStream::stderr(ColorChoice::Always);
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
            .expect("cannot set color for stderr in StandardStream");
        writeln!(&mut stderr, "{}", self).expect("cannot write to stderr");

        std::process::exit(0)
    }
}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CliError::Generic(msg) => format!("Error: {}", msg),
                CliError::DestinationAlreadyExists(path) =>
                    format!("Error: destination {} already exists", path.display()),
                CliError::PathNotValid(path) => {
                    format!("Error: {} is not a valid path", path.display())
                }
                CliError::ProofNotValid(hex_error) => {
                    format!("Error: proof is invalid ({})", hex_error)
                }
            }
        )
    }
}

impl From<InputParserError> for CliError {
    fn from(error: InputParserError) -> Self {
        CliError::Generic(error.to_string())
    }
}
