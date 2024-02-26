mod email;
mod name;

use std::fmt::Display;

pub use email::SubscriberEmail;
pub use name::SubscriberName;

use crate::routes::FormData;

#[derive(Debug, Clone)]
pub struct Subscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}

impl Display for Subscriber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} <{}>", self.name, self.email)
    }
}

impl TryFrom<FormData> for Subscriber {
    type Error = String;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let email = SubscriberEmail::parse(form.email)?;
        let name = SubscriberName::parse(form.name)?;
        Ok(Self { email, name })
    }
}
