use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "cli")]
use clap::builder::{PossibleValue, Str};

#[cfg(feature = "cli")]
use clap::ValueEnum;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthType {
    #[default]
    PublicKey,

    /// AuthenticationAgent is a placeholder authentication type.
    /// It's not recognized by the `ssh` command (as a valid `PreferredAuthentications` choice),
    /// but instructs us to avoid specifying a preferred authentication and SSH key (`ssh -i ..`),
    /// thus delegating to an SSH agent, if available.
    AuthenticationAgent,

    Password,

    KeyboardInteractive,

    HostBased,

    GSSAPIWithMic,
}

impl AuthType {
    pub const ALL: [AuthType; 6] = [
        Self::PublicKey,
        Self::AuthenticationAgent,
        Self::Password,
        Self::KeyboardInteractive,
        Self::HostBased,
        Self::GSSAPIWithMic,
    ];

    pub fn to_static_str(&self) -> &'static str {
        match &self {
            Self::PublicKey => "publickey",
            Self::AuthenticationAgent => "authentication-agent",
            Self::Password => "password",
            Self::KeyboardInteractive => "keyboard-interactive",
            Self::HostBased => "hostbased",
            Self::GSSAPIWithMic => "gssapi-with-mic",
        }
    }

    pub fn from_string(s: &str) -> Result<Self, &'static str> {
        match s {
            "publickey" => Ok(Self::PublicKey),
            "authentication-agent" => Ok(Self::AuthenticationAgent),
            "password" => Ok(Self::Password),
            "keyboard-interactive" => Ok(Self::KeyboardInteractive),
            "hostbased" => Ok(Self::HostBased),
            "gssapi-with-mic" => Ok(Self::GSSAPIWithMic),
            _ => Err("Unexpected string value"),
        }
    }
}

impl std::fmt::Display for AuthType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{0}", self.to_static_str())
    }
}

#[cfg(feature = "cli")]
impl ValueEnum for AuthType {
    fn value_variants<'a>() -> &'a [Self] {
        &AuthType::ALL
    }

    #[cfg(feature = "cli")]
    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(PossibleValue::new(Str::from(self.to_static_str())))
    }
}

// Custom serialization for AuthType
pub fn serialize_auth_type_to_string<S>(value: &AuthType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(value.to_static_str())
}

// Custom deserialization for AuthType
pub fn deserialize_auth_type_from_string<'de, D>(deserializer: D) -> Result<AuthType, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    AuthType::from_string(&s).map_err(DeError::custom)
}
