use std::fmt::Display;

#[derive(Clone, Debug, Hash, Default, PartialEq, Eq)]
pub struct Chown {
    pub user: Option<String>,
    pub group: Option<String>,
}

impl Chown {
    pub fn new(user: impl AsRef<str>, group: impl AsRef<str>) -> Self {
        Self {
            user: Some(user.as_ref().to_owned()),
            group: Some(group.as_ref().to_owned()),
        }
    }

    pub fn user(user: impl AsRef<str>) -> Self {
        Self {
            user: Some(user.as_ref().to_owned()),
            group: None,
        }
    }

    pub fn group(group: impl AsRef<str>) -> Self {
        Self {
            user: None,
            group: Some(group.as_ref().to_owned()),
        }
    }
}

impl Display for Chown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(user) = &self.user {
            write!(f, "{user}")?;
        }
        if let Some(grp) = &self.group {
            write!(f, ":{grp}")?;
        }
        Ok(())
    }
}
