use chrono::{
    NaiveDate,
    format::{Parsed, StrftimeItems},
};

use super::{NetflowDetailItemRes, NetflowDetailRes};
use hnu_query::netflow::detail::Detail as SpiderNetflowDetail;

/// 解析`%Y-%m`格式的字符串，将其转为当月的第一天。
pub fn parse_year_month(str: &str) -> Option<NaiveDate> {
    let mut parsed = Parsed::new();
    chrono::format::parse(
        &mut parsed,
        str,
        StrftimeItems::new("%Y-%m"),
    )
    .ok()?;
    parsed.set_day(1).ok()?;
    parsed.to_naive_date().ok()
}

/// 将字节转为 GB
pub fn bytes_to_gb(bytes: f64) -> String {
    if bytes == 0.0 {
        "0 GB".to_string()
    } else {
        format!("{:.2} GB", bytes / 1024.0 / 1024.0 / 1024.0)
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
        assert_eq!(parse_year_month("2077-13"), None);
    }
}
