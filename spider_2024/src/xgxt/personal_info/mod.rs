mod dormitory;
mod raw;

use crate::xgxt::personal_info::{
    dormitory::parse_dormitory, raw::raw_person_info_data,
};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};

pub use dormitory::Dormitory;

#[derive(Debug, Serialize, Deserialize)]
pub struct PersonalInfo {
    /// 姓名
    pub name: String,
    /// 年级（入学年份应该与年级相等），如 `2024`
    pub enter_year: u16,
    /// 学制，如 `4`
    pub xz: u8,
    /// 学号
    pub stu_id: String,
    /// 性别
    pub gender: Gender,
    /// 培养层次，区分本科/研究生/博士生
    ///
    /// TODO 目前这个字段只有数字字符串，后续需要进一步解析
    pub level: String,
    /// 学院
    ///
    /// TODO 目前这个字段只有数字字符串，后续需要进一步解析
    pub academy: String,
    /// 专业
    ///
    /// TODO 目前这个字段只有数字字符串，后续需要进一步解析
    pub major: String,
    /// 班级
    ///
    /// TODO 目前这个字段只有数字字符串，后续需要进一步解析
    pub class: String,
    /// 宿舍信息
    pub dormitory: Dormitory,
    /// 政治面貌
    ///
    /// TODO 目前这个字段只有数字字符串，后续需要进一步解析
    pub politic: Option<String>,
    /// 民族
    ///
    /// TODO 目前这个字段只有数字字符串，后续需要进一步解析
    pub race: Option<String>,
    /// 籍贯
    ///
    /// TODO 目前这个字段只有以逗号分割的数字字符串，后续需要进一步解析
    pub hometown: Option<String>,
    /// 手机号
    pub phone: Option<String>,
    /// 微信号
    pub wechat: Option<String>,
    /// qq号
    pub qq: Option<String>,
    /// 电子邮箱
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Gender {
    Male,
    Female,
}

/// 从学工系统获取个人信息
///
/// # Parameters
///
/// - `stu_id`: 学号
///
/// # Returns
///
/// 个人信息
///
/// # Performance
///
/// 这个函数大概会同时发起三个请求，且一次请求数据量比较大（学工系统有个接口直接把近十年所有的班级数据全部返回了），所以建议不要频繁调用本函数。个人信息一般没有什么变动，建议做好缓存。
pub async fn get_person_info(
    stu_id: &str,
) -> Result<PersonalInfo, crate::Error> {
    let mut entries = raw_person_info_data(stu_id).await?;

    let name =
        entries.remove("姓名").ok_or(anyhow!("无法找到姓名信息"))?;
    let enter_year: u16 = entries
        .remove("年级")
        .ok_or(anyhow!("无法找到入学年份信息"))?
        .parse()
        .map_err(|e| anyhow!("无法解析入学年份信息: {}", e))?;
    let xz: u8 = entries
        .remove("学制(年)")
        .ok_or(anyhow!("无法找到学制信息"))?
        .parse()
        .map_err(|e| anyhow!("无法解析学制信息: {}", e))?;
    let stu_id =
        entries.remove("学号").ok_or(anyhow!("无法找到学号信息"))?;
    let gender = match entries.get("性别").map(|v| v.as_str()) {
        Some("1") => Gender::Male,
        Some("2") => Gender::Female,
        v => {
            return Err(anyhow!("解析性别失败, data: {:?}", v).into());
        }
    };
    let level = entries
        .remove("培养层次")
        .ok_or(anyhow!("无法找到培养层次信息"))?;
    let academy =
        entries.remove("学院").ok_or(anyhow!("无法找到学院信息"))?;
    let major =
        entries.remove("专业").ok_or(anyhow!("无法找到专业信息"))?;
    let class =
        entries.remove("班级").ok_or(anyhow!("无法找到班级信息"))?;
    let dormitory = parse_dormitory(
        entries
            .remove("寝室楼")
            .ok_or(anyhow!("无法找到寝室楼信息"))?,
        entries
            .remove("寝室号")
            .ok_or(anyhow!("无法找到寝室号信息"))?,
    );

    let res = PersonalInfo {
        name,
        enter_year,
        xz,
        stu_id,
        gender,
        level,
        academy,
        major,
        class,
        dormitory,
        politic: entries.remove("政治面貌"),
        race: entries.remove("民族"),
        hometown: entries.remove("籍贯"),
        phone: entries.remove("手机号码"),
        wechat: entries.remove("微信号"),
        qq: entries.remove("QQ号码"),
        email: entries.remove("电子邮箱"),
    };
    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::TEST_STU_ID;

    #[tokio::test]
    async fn test_get_person_info() {
        dbg!(get_person_info(&TEST_STU_ID).await.unwrap());
    }
}
