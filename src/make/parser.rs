use std::collections::{BTreeSet, HashMap};
use std::fmt::{self, Display};
use std::marker::PhantomData;
use std::str::FromStr;

use raptor_parser::util::module_name::ModuleName;
use serde::de::{DeserializeOwned, MapAccess, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug)]
pub struct Make {
    pub raptor: Raptor,
    pub run: HashMap<String, RunTarget>,
    pub group: HashMap<String, GroupTarget>,
}

#[derive(Deserialize, Debug)]
pub struct Raptor {
    #[serde(deserialize_with = "de_map_string_or_struct")]
    pub link: HashMap<String, Link>,
}

#[derive(Deserialize, Debug)]
pub struct Link {
    pub source: String,
}

#[derive(Deserialize, Debug)]
pub struct GroupTarget {
    pub run: BTreeSet<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct RunTarget {
    #[serde(with = "module_name")]
    pub target: ModuleName,

    #[serde(deserialize_with = "string_or_list", default)]
    pub cache: Vec<String>,

    #[serde(deserialize_with = "string_or_list", default)]
    pub input: Vec<String>,

    #[serde(default)]
    pub output: Option<String>,

    #[serde(default)]
    pub entrypoint: Vec<String>,

    #[serde(default)]
    pub state_dir: Option<String>,

    #[serde(default)]
    pub args: Vec<String>,

    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl FromStr for Link {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let source = s.to_string();

        Ok(Self { source })
    }
}

#[derive(Debug, Clone)]
pub enum MakeTarget {
    Job(String),
    Group(String),
}

impl FromStr for MakeTarget {
    type Err = &'static str;

    #[allow(clippy::option_if_let_else)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res = if let Some(group) = s.strip_prefix('%') {
            Self::Group(group.to_string())
        } else {
            Self::Job(s.to_string())
        };

        Ok(res)
    }
}

pub mod module_name {
    use raptor_parser::util::module_name::ModuleName;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ModuleName, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = String::deserialize(deserializer)?;

        Ok(ModuleName::from(val.as_str()))
    }

    pub fn serialize<S>(name: &ModuleName, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{name}");
        serializer.serialize_str(&s)
    }
}

pub fn string_or_list<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: Deserialize<'de> + FromStr,
    D: Deserializer<'de>,
    T::Err: Display,
{
    use serde::de::value::SeqAccessDeserializer;
    use serde::de::{Error, SeqAccess};

    struct StringOrList<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for StringOrList<T>
    where
        T: Deserialize<'de> + FromStr,
        T::Err: Display,
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list")
        }

        fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
            let value = T::from_str(value).map_err(Error::custom)?;

            Ok(vec![value])
        }

        fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error> {
            Deserialize::deserialize(SeqAccessDeserializer::new(seq))
        }
    }

    deserializer.deserialize_any(StringOrList(PhantomData))
}

pub fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr,
    D: Deserializer<'de>,
    T::Err: Display,
{
    use serde::de::Error;

    struct StringOrStruct<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr,
        T::Err: Display,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list")
        }

        fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
            let value = T::from_str(value).map_err(Error::custom)?;

            Ok(value)
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let _ = map;
            Err(Error::invalid_type(Unexpected::Map, &self))
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}

pub fn de_map_string_or_struct<'de, D, T>(deserializer: D) -> Result<HashMap<String, T>, D::Error>
where
    D: Deserializer<'de>,
    T: for<'x> Deserialize<'x> + FromStr,
    T::Err: Display,
{
    #[derive(Deserialize)]
    struct Wrapper<T>(#[serde(deserialize_with = "string_or_struct")] T)
    where
        T: DeserializeOwned + FromStr,
        T::Err: Display;

    let v = HashMap::<String, Wrapper<T>>::deserialize(deserializer)?;
    Ok(v.into_iter().map(|(k, Wrapper(v))| (k, v)).collect())
}
