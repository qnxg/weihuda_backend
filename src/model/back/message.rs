use chrono::{DateTime, Utc};
use serde::{ser::Serializer, Serialize};
use sqlx::FromRow;

#[derive(FromRow, Serialize, Debug)]
pub struct MessageInfo {
    #[serde(rename = "create_at", serialize_with = "serialize_as_date")]
    pub created_at: Option<DateTime<Utc>>, //将数据库默认时区设置为东八区，重新调试。我无法在这里直接使用DateTime<Local>，因为sqlx::FromRow要求Option<DateTime<Utc>>，好像也只为这个类型实现了转换，后续在Github查询一下资料。// 在serde为json返回值的时候重新转换回Local类型，不清楚前端需求，后续再改
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
            let s = date.to_string().split(' ').next().unwrap_or("").to_string();
            serializer.serialize_str(&s)
        }
        None => serializer.serialize_none(),
    }
}

// fn serialize_as_local<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
// where
//     S: Serializer,
// {
//     match date {
//         Some(date) => {
//             let local: DateTime<Local> = DateTime::<Local>::from(*date);
//             serializer.serialize_str(&local.to_rfc3339())
//         }
//         None => serializer.serialize_none(),
//     }
// }
