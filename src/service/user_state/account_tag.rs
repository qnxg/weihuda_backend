use moka::future::Cache;
use std::{sync::LazyLock, time::Duration};

use crate::result::AppError;

pub static ACCOUNT_TAG: LazyLock<Cache<String, AppError>> =
    LazyLock::new(|| {
        Cache::builder()
            .time_to_live(Duration::from_mins(1))
            .max_capacity(1000)
            .build()
    });
