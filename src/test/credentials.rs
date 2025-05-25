use std::env;

use crate::connect_options::Credentials;

#[must_use]
pub fn credentials() -> Option<(String, String)> {
    let user = env::var("MQ_USER").ok()?;
    let password = env::var("MQ_PASSWORD").ok()?;
    Some((user, password))
}

impl<'cred> From<Option<&'cred (String, String)>> for Credentials<'cred, &'cred str> {
    fn from(value: Option<&'cred (String, String)>) -> Self {
        match value {
            Some((user, password)) => Credentials::User(user.as_str(), password.as_str().into()),
            None => Credentials::Default,
        }
    }
}
