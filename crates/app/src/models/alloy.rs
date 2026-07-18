use std::{fmt::Display, ops::Deref, str::FromStr};

use alloy::primitives::{Address, Bytes, U256};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

macro_rules! api_string_type {
    (
        $(#[$derive:meta])*
        pub struct $name:ident($inner:ty);
        type_name: $type_name:literal;
        format: $format:literal;
        pattern: $pattern:literal;
        parse_error: $parse_error:ty;
        error_message: $error_message:literal;
    ) => {
        $(#[$derive])*
        pub struct $name(pub $inner);

        impl Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $inner {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Display::fmt(&self.0, f)
            }
        }

        impl FromStr for $name {
            type Err = $parse_error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <$inner>::from_str(s).map(Self)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.to_string())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                value.parse().map_err(D::Error::custom)
            }
        }

    };
}

api_string_type! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ApiAddress(Address);
    type_name: "Address";
    format: "address";
    pattern: "^0x[a-fA-F0-9]{40}$";
    parse_error: alloy::primitives::hex::FromHexError;
    error_message: "expected address string";
}

api_string_type! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ApiU256(U256);
    type_name: "U256";
    format: "uint256";
    pattern: "^[0-9]+$";
    parse_error: alloy::primitives::ruint::ParseError;
    error_message: "expected U256 string";
}

api_string_type! {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ApiBytes(Bytes);
    type_name: "Bytes";
    format: "bytes";
    pattern: "^0x([a-fA-F0-9]{2})*$";
    parse_error: alloy::primitives::hex::FromHexError;
    error_message: "expected bytes string";
}
