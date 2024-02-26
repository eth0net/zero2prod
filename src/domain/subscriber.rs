mod email;
mod name;

pub use email::SubscriberEmail;
pub use name::SubscriberName;

use crate::routes::FormData;

pub struct Subscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}

impl TryFrom<FormData> for Subscriber {
    type Error = String;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let email = SubscriberEmail::parse(form.email)?;
        let name = SubscriberName::parse(form.name)?;
        Ok(Self { email, name })
    }
}
