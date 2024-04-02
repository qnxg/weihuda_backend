use chrono::{DateTime, Utc};
use serde::{ser::Serializer, Serialize};

#[derive(Serialize, Debug)]
pub struct MessageInfo {
    #[serde(rename = "create_at", serialize_with = "serialize_as_date")]
    pub created_at: Option<DateTime<Utc>>,
    pub url: Option<String>,
    pub title: String,
    pub content: String,
    pub id: u32,
}

fn serialize_as_date<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match date {
        Some(date) => {
            let s = date
                .with_timezone(&chrono::FixedOffset::east_opt(8 * 3600).unwrap())
                .naive_local()
                .to_string()
                .split(' ')
                .next()
                .unwrap_or("")
                .to_string();
            serializer.serialize_str(&s)
        }
        None => serializer.serialize_none(),
    }
}
