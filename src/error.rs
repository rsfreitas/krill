pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InternalOS(String),
    DefinitionParser(String),
    UnsupportedSetting(String),
}

impl Error {
    fn description(&self) -> String {
        match &*self {
            Error::InternalOS(s) => format!("internal Operating System error '{}'", s),
            Error::DefinitionParser(s) => {
                format!("failed to parse service definitions '{}'", s)
            }
            Error::UnsupportedSetting(s) => format!("unsupported toml setting '{}'", s),
        }
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}
