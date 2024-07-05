use crate::model;


// Result
pub type Result<T> = core::result::Result<T, Error>;


// Error
#[derive(Debug)]
pub enum Error {
	ConfigMissingEnv(&'static str),
	ConfigWrongFormat(&'static str),
	Model(model::Error)
}

impl From<model::Error> for Error {
	fn from(val: model::Error) -> Self {
		Self::Model(val)
	}
}




impl core::fmt::Display for Error {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}