use crate::Marker;
use std::convert::From;
use std::string::{FromUtf8Error, String};
use std::{fmt, io};

/// The error type for serde operations of the [`to_bytes`] and [`from_bytes`] methods.
///
/// [`to_bytes`]: crate::to_bytes
/// [`from_bytes`]: crate::from_bytes
#[derive(Debug)]
pub struct Error {
    kind: Kind,
}

#[derive(Debug)]
enum Kind {
    NoLength,
    UnexpectedBool { value: u8 },
    LengthMismatch { expected: usize, found: usize },
    LengthTooLong,
    MissingHeaderBytes,
    InvalidFieldName(FromUtf8Error),
    UnknownMarker { value: Marker },
    Io(io::Error),
    Custom(String),
    RootMustBeStruct { value: Marker },
    F32IsNotSupported,
    NoneCanNotBeSerialized,
    UnitIsNotSupported,
    EnumsAreNotSupported,
    TuplesOfTypeAreNotSupported { marker: Marker },
    TupleStructsAreNotSupported,
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error::custom(msg)
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error::custom(msg)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            Kind::InvalidFieldName(inner) => Some(inner),
            Kind::Io(inner) => Some(inner),
            _ => None,
        }
    }
}

impl Error {
    fn custom<T: ToString>(msg: T) -> Self {
        Self {
            kind: Kind::Custom(msg.to_string()),
        }
    }

    pub(crate) fn missing_header_bytes() -> Self {
        Self {
            kind: Kind::MissingHeaderBytes,
        }
    }

    pub(crate) fn unexpected_bool(value: u8) -> Self {
        Self {
            kind: Kind::UnexpectedBool { value },
        }
    }

    pub(crate) fn length_mismatch(expected: usize, found: usize) -> Self {
        Self {
            kind: Kind::LengthMismatch { expected, found },
        }
    }

    pub(crate) fn unknown_marker(value: Marker) -> Self {
        Self {
            kind: Kind::UnknownMarker { value },
        }
    }

    pub(crate) fn f32_is_not_supported() -> Self {
        Self {
            kind: Kind::F32IsNotSupported,
        }
    }

    pub(crate) fn no_length() -> Self {
        Self {
            kind: Kind::NoLength,
        }
    }

    pub(crate) fn root_must_be_struct(marker: Marker) -> Error {
        Self {
            kind: Kind::RootMustBeStruct { value: marker },
        }
    }

    pub(crate) fn enums_are_not_supported() -> Error {
        Self {
            kind: Kind::EnumsAreNotSupported,
        }
    }

    pub(crate) fn tuples_of_type_are_not_supported(marker: Marker) -> Error {
        Self {
            kind: Kind::TuplesOfTypeAreNotSupported { marker },
        }
    }

    pub(crate) fn tuple_structs_are_not_supported() -> Error {
        Self {
            kind: Kind::TupleStructsAreNotSupported,
        }
    }

    pub(crate) fn unit_is_not_supported() -> Error {
        Self {
            kind: Kind::UnitIsNotSupported,
        }
    }

    pub(crate) fn none_can_not_be_serialized() -> Error {
        Self {
            kind: Kind::NoneCanNotBeSerialized,
        }
    }

    pub(crate) fn length_exceeded_max_size() -> Error {
        Self {
            kind: Kind::LengthTooLong,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            Kind::NoLength => write!(f, "Length of seq/map must be known ahead of time"),
            Kind::UnexpectedBool { value } => write!(f, "{} is not a valid boolean value", value),
            Kind::MissingHeaderBytes => write!(f, "Missing magic header bytes"),
            Kind::InvalidFieldName(_) => write!(f, "Fieldname contained non-UTF-8 characters"),
            Kind::UnknownMarker { value } => write!(f, "Unknown marker value {}", value),
            Kind::Io(_) => write!(f, "Failed to read from or write to buffer"),
            Kind::Custom(inner) => write!(f, "{}", inner),
            Kind::RootMustBeStruct { value } => {
                write!(f, "Root element must be a struct but got {}", value)
            }
            Kind::F32IsNotSupported => write!(f, "Type f32 is not supported"),
            Kind::NoneCanNotBeSerialized => write!(f, "Optional fields must be wrapped in #[serde(skip_serializing_if = \"Option::is_none\")]"),
            Kind::UnitIsNotSupported => write!(f, "Unit type is not supported"),
            Kind::EnumsAreNotSupported => write!(f, "Enums are not supported"),
            Kind::TuplesOfTypeAreNotSupported { marker } => {
                write!(f, "Tuples of type {} are not supported", marker)
            }
            Kind::TupleStructsAreNotSupported => write!(f, "Tuple structs are not supported"),
            Kind::LengthMismatch { expected, found } => write!(
                f,
                "Length mismatch, expected {} elements but found {}",
                expected, found
            ),
            Kind::LengthTooLong => write!(f, "Length of field exceeded maximum size"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error { kind: Kind::Io(e) }
    }
}

impl From<FromUtf8Error> for Error {
    fn from(e: FromUtf8Error) -> Self {
        Error {
            kind: Kind::InvalidFieldName(e),
        }
    }
}
