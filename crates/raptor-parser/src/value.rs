use std::collections::BTreeMap;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum Value {
    Bool(bool),
    Number(i64),
    String(String),
    List(Vec<Value>),
    Map(BTreeMap<Value, Value>),
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Number(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl<const N: usize, T: Into<Self>> From<[T; N]> for Value {
    fn from(value: [T; N]) -> Self {
        Self::List(value.map(Into::into).to_vec())
    }
}

impl<K: Into<Self>, V: Into<Self>> From<BTreeMap<K, V>> for Value {
    fn from(value: BTreeMap<K, V>) -> Self {
        Self::Map(
            value
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }
}
