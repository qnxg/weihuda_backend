use super::{NetflowDetailItemRes, NetflowDetailRes};
use crate::error::{AppResult, ThrowInternalErrorResult};
use chrono::{
    NaiveDate,
    format::{Parsed, StrftimeItems},
};

use hnu_query::netflow::detail::Detail as SpiderNetflowDetail;

/// 解析`%Y-%m`格式的字符串，将其转为当月的第一天。
pub fn parse_year_month(str: &str) -> AppResult<NaiveDate> {
    let mut parsed = Parsed::new();
    chrono::format::parse(
        &mut parsed,
        str,
        StrftimeItems::new("%Y-%m"),
    )
    .internal_err()?;
    parsed.set_day(1).internal_err()?;
    parsed.to_naive_date().internal_err()
}

/// 将字节转为 GB
pub fn bytes_to_gb(bytes: usize) -> String {
    if bytes == 0 {
        "0 GB".to_string()
    } else {
        format!("{:.2} GB", bytes / 1024 / 1024 / 1024)
    }
}

pub fn convert_netflow_detail(
    detail: SpiderNetflowDetail,
) -> NetflowDetailRes {
    NetflowDetailRes {
        AllDownload: detail.download,
        AllTotal: detail.total,
        AllUpload: detail.upload,
        FloatDetailList: detail
            .items
            .into_iter()
            .map(|item| NetflowDetailItemRes {
                App: item.app,
                Download: item.download,
                Per: item.percentage,
                Total: item.total,
                Upload: item.upload,
            })
            .collect(),
    }
}

#[cfg(test)]
mod test {
    use super::parse_year_month;
    #[test]
    fn test_parse_year_month() {
        assert_eq!(
            parse_year_month("2025-01").unwrap(),
            "2025-01-01".parse().unwrap()
        );
        assert_eq!(
            parse_year_month("2077-12").unwrap(),
            "2077-12-01".parse().unwrap()
        );
        assert_eq!(
            parse_year_month("2077-3").unwrap(),
            "2077-03-01".parse().unwrap()
        );
        assert!(parse_year_month("2077-13").is_err());
    }
}
