#![allow(unused)] // For beginning only.

pub type Rresult<T> = core::result::Result<T, Error>;
use hmac::{digest::InvalidLength};

// Error
#[allow(non_snake_case, non_camel_case_types)]
#[derive(Debug)]
pub enum Error {
	HMAC_SHA512(&'static str),
	HmacInvalidLength(InvalidLength)
    
}


impl From<InvalidLength> for Error {
	fn from(val: InvalidLength) -> Self {
		Self::HmacInvalidLength(val)
	}
}