mod error;
pub mod repo;
pub mod scanner;

pub use error::Error;
pub use repo::Repository;
pub use scanner::Scanner;

/// A typedef of the result returned by many methods.
pub type Result<T, E = Error> = std::result::Result<T, E>;
