use diesel::{
    deserialize::{self, FromSql},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Text,
    sqlite::{Sqlite, SqliteValue},
    AsExpression, FromSqlRow,
};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str::FromStr};

macro_rules! db_text_enum {
    (
        $name:ident {
            $($variant:ident => $value:literal),+ $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, AsExpression, FromSqlRow)]
        #[diesel(sql_type = Text)]
        pub(crate) enum $name {
            $($variant),+
        }

        impl $name {
            pub(crate) fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $value),+
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                value.parse::<Self>().map_err(de::Error::custom)
            }
        }

        impl FromStr for $name {
            type Err = String;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    $($value => Ok(Self::$variant),)+
                    _ => Err(format!("invalid {}: {value}", stringify!($name))),
                }
            }
        }

        impl ToSql<Text, Sqlite> for $name {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
                out.set_value(self.as_str());
                Ok(IsNull::No)
            }
        }

        impl FromSql<Text, Sqlite> for $name {
            fn from_sql(mut value: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
                value.read_text().parse::<Self>().map_err(Into::into)
            }
        }
    };
}

db_text_enum! {
    SessionKind {
        Researcher => "researcher",
        Coder => "coder",
        Reviewer => "reviewer",
        Openclaw => "openclaw",
    }
}

db_text_enum! {
    SessionStatus {
        Idle => "idle",
        Moving => "moving",
        Working => "working",
        Error => "error",
        Archived => "archived",
    }
}

db_text_enum! {
    Direction {
        Up => "up",
        Left => "left",
        Down => "down",
        Right => "right",
    }
}

db_text_enum! {
    MessageRole {
        User => "user",
        Assistant => "assistant",
        System => "system",
    }
}

db_text_enum! {
    MessageStatus {
        Pending => "pending",
        Streaming => "streaming",
        Complete => "complete",
        Error => "error",
    }
}
