use super::get_db_pool;
use crate::{result::AppResult, utils};
use anyhow::anyhow;

const SQL_START_DATE_TABLE_KEY: &str = "classStartDateTable";
const SQL_VACATION_DATE_KEY: &str = "nextVacationDate";

/// JSON格式为：
/// `[["xxxx-n", "yyyy-mm-dd"], ...]`
/// 每一项的前一项为`学年-学期`，后一项为学期开始日期。
/// 数字位数要确认相同。
pub async fn get_class_start_date_table()
-> AppResult<Vec<(String, String)>> {
    let table_json: String = sqlx::query!(
        r#"
            SELECT
                value
            FROM
                mini_configs
            WHERE
                `key` = ? AND enabled = 1
            "#,
        SQL_START_DATE_TABLE_KEY
    )
    .fetch_one(get_db_pool().await)
    .await
    .map_err(|_| anyhow!("学期日期表不见了"))?
    .value;

    let table: Vec<(String, String)> =
        serde_json::from_str(&table_json)
            .map_err(|e| anyhow!("解析学期表 JSON 失败：{}", e))?;
    for (xnxq, date) in &table {
        assert!(utils::time::is_well_formed_xnxq(xnxq));
        assert!(utils::time::is_well_formed_date(date));
    }

    Ok(table)
}

/// 获取下一假期时间
pub async fn get_vacation_date() -> AppResult<String> {
    let res = sqlx::query!(
        r#"
                SELECT
                    value
                FROM
                    mini_configs
                WHERE
                    `key` = ? AND enabled = 1
                "#,
        SQL_VACATION_DATE_KEY
    )
    .fetch_one(get_db_pool().await)
    .await
    .map_err(|_| anyhow!("假期时间数据不见了"))?
    .value;

    assert!(utils::time::is_well_formed_date(&res));
    Ok(res)
}
