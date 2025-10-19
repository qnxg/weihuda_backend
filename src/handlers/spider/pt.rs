use axum::Extension;

use crate::{
    app_result::AppResult,
    dtos::spider::{
        pt::{GetCardHistoryReq, GetFitnessReq},
        xgxt::PersonInfo,
    },
    entities::spider::{
        card::{
            CardHistoryRes, CardHistoryResItem, CardInfoRes,
            SpiderCardHistory, SpiderCardInfo,
        },
        email::SpiderEmail,
        fitness::{
            get_class_color, FitnessAppointRes, FitnessRes,
            FitnessResEye, FitnessResItem, FitnessResReport,
            FitnessResStudent, FitnessResTotal, SpiderFitness,
            SpiderFitnessAppoint, SpiderFitnessRaw,
        },
    },
    extractors::Query,
    utils::{
        jwt::parse_stu_id,
        request::{spider, spider_data},
    },
};

/// 获取校园一卡通信息
pub async fn get_card_info_handler(
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("stuid", stu_id)];
    let spider_res: SpiderCardInfo =
        spider_data("/pt/card/info", &params).await?;

    let res = CardInfoRes {
        account: spider_res.account,
        balance: spider_res.balance.parse::<f64>().unwrap() / 100.0,
    };

    Ok(res.into())
}

/// 获取校园一卡通消费历史
pub async fn get_card_history_handler(
    Query(req): Query<GetCardHistoryReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [
        ("stuid", stu_id),
        ("year", req.year),
        ("month", req.month),
        ("type", req._type),
    ];
    let spider_res: SpiderCardHistory =
        spider_data("/pt/card/history", &params).await?;

    let mut res_items = Vec::with_capacity(spider_res.items.len());
    for item in spider_res.items {
        let item = CardHistoryResItem {
            tranAmt: item.fTranAmt,
            effectDate: item.effectdate,
            jourDate: item.jndatetime,
            jourName: item.jourName,
            jourNum: item.usedcardnum,
            nowAmt: item.nowAmt,
            tranLocation: item.sysname1.unwrap_or_default(),
            tranname: item.tranname,
        };
        res_items.push(item);
    }
    let res = CardHistoryRes {
        TranCount: spider_res.TranCount,
        total: spider_res.total,
        items: res_items,
    };
    Ok(res.into())
}

/// 获取校园邮箱未读邮件数
pub async fn get_email_handler(
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("stuid", stu_id)];
    let spider_res: SpiderEmail =
        match spider_data("/pt/email", &params).await {
            Ok(res) => res,
            Err(_) => return Ok(serde_json::Value::Null.into()),
        };

    match spider_res.unReadCount {
        Some(count) => Ok(count.into()),
        None => Ok(serde_json::Value::Null.into()),
    }
}

/// 获取体测成绩
#[expect(clippy::too_many_lines, reason = "REFACTOR ME")]
pub async fn get_fitness_handler(
    Query(req): Query<GetFitnessReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;

    let param_stuid_xn = &[("stuid", &stu_id), ("xn", &req.xn)];
    let param_stuid = &[("stuid", &stu_id), ("xn", &req.xn)];

    let (grade, raw_grade, person_info) = tokio::try_join!(
        async {
            spider::<_, SpiderFitness>("/gymos/grade", param_stuid_xn)
                .await
                .map_err(|e| e.into())
        },
        spider_data::<_, SpiderFitnessRaw>(
            "/gymos/raw_grade",
            param_stuid_xn,
        ),
        spider_data::<_, PersonInfo>(
            "/xgxt/person_info",
            param_stuid,
        )
    )?;

    let res = FitnessRes {
        student: FitnessResStudent {
            name: raw_grade.student_name,
            number: raw_grade.student_num,
        },
        total: FitnessResTotal {
            score: raw_grade.total_score,
            grade: raw_grade.total_grade,
        },
        report: FitnessResReport {
            desc: raw_grade.report_desc,
            status: grade
                .report_status
                .unwrap_or(raw_grade.status.to_string()),
            _type: raw_grade.report_type.to_string(),
        },
        eye: FitnessResEye {
            eyesight_right: format!(
                "{} {}",
                raw_grade.eyesight_right,
                raw_grade.eyesight_right_detail
            ),
            eyesight_left: format!(
                "{} {}",
                raw_grade.eyesight_left,
                raw_grade.eyesight_left_detail
            ),
            eye_mirror_right: format!(
                "{} {}",
                raw_grade.eye_mirror_right,
                raw_grade.eye_mirror_right_detail
            ),
            eye_mirror_left: format!(
                "{} {}",
                raw_grade.eye_mirror_left,
                raw_grade.eye_mirror_left_detail
            ),
            eye_ametropia_right: format!(
                "{} {}",
                raw_grade.eye_ametropia_right,
                raw_grade.eye_ametropia_right_detail
            ),
            eye_ametropia_left: format!(
                "{} {}",
                raw_grade.eye_ametropia_left,
                raw_grade.eye_ametropia_left_detail
            ),
        },
        items: vec![
            FitnessResItem {
                name: "50m".to_string(),
                class: grade.data.short_run_class.unwrap_or(
                    get_class_color(&raw_grade.short_run_grade),
                ),
                score: grade
                    .data
                    .short_run_score
                    .unwrap_or(raw_grade.short_run + "秒"),
                rank: raw_grade.short_run_grade,
                grade: raw_grade.short_run_score,
            },
            FitnessResItem {
                name: "BMI".to_string(),
                class: grade
                    .data
                    .bmi_class
                    .unwrap_or(get_class_color(&raw_grade.bmi_grade)),
                score: grade.data.bmi_score.unwrap_or(format!(
                    "{}厘米/{}千克",
                    raw_grade.height, raw_grade.weight
                )),
                rank: raw_grade.bmi_grade,
                grade: raw_grade.bmi_score,
            },
            FitnessResItem {
                name: "跳远".to_string(),
                class: grade.data.jump_class.unwrap_or(
                    get_class_color(&raw_grade.jump_grade),
                ),
                score: grade
                    .data
                    .jump_score
                    .unwrap_or(raw_grade.jump + "厘米"),
                rank: raw_grade.jump_grade,
                grade: raw_grade.jump_score,
            },
            FitnessResItem {
                name: if person_info.gender == "男" {
                    "引体向上"
                } else {
                    "仰卧起坐"
                }
                .to_string(),
                class: grade.data.pull_and_sit_class.unwrap_or(
                    get_class_color(&raw_grade.pull_and_sit_grade),
                ),
                score: grade
                    .data
                    .pull_and_sit_score
                    .unwrap_or(raw_grade.pull_and_sit.to_string()),
                rank: raw_grade.pull_and_sit_grade,
                grade: raw_grade.pull_and_sit_score
                    + raw_grade.extra_score_pull_or_sit_up,
            },
            FitnessResItem {
                name: "长跑".to_string(),
                class: grade
                    .data
                    .run_class
                    .unwrap_or(get_class_color(&raw_grade.run_grade)),
                score: grade.data.run_score.unwrap_or({
                    let total_seconds: u32 =
                        raw_grade.run.parse().unwrap_or(0);
                    let minutes = total_seconds / 60;
                    let seconds = total_seconds - minutes * 60;
                    if seconds != 0 {
                        format!("{}'{}\"", minutes, seconds)
                    } else {
                        format!("{}'", minutes)
                    }
                }),
                rank: raw_grade.run_grade,
                grade: raw_grade.run_score
                    + raw_grade.extra_score_run,
            },
            FitnessResItem {
                name: "坐位体前屈".to_string(),
                class: grade.data.sit_and_reach_class.unwrap_or(
                    get_class_color(&raw_grade.sit_and_reach_grade),
                ),
                score: grade
                    .data
                    .sit_and_reach_score
                    .unwrap_or(raw_grade.sit_and_reach + "厘米"),
                rank: raw_grade.sit_and_reach_grade,
                grade: raw_grade.sit_and_reach_score,
            },
            FitnessResItem {
                name: "肺活量".to_string(),
                class: grade
                    .data
                    .vc_class
                    .unwrap_or(get_class_color(&raw_grade.vc_grade)),
                score: grade
                    .data
                    .vc_score
                    .unwrap_or(raw_grade.vc.to_string() + "毫升"),
                rank: raw_grade.vc_grade,
                grade: raw_grade.vc_score,
            },
        ],
    };
    Ok(res.into())
}

pub async fn get_fitness_appoint_handler(
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("stuid", stu_id)];
    let spider_res: Vec<SpiderFitnessAppoint> =
        spider_data("/gymos/appoint", &params).await?;

    let mut res = Vec::with_capacity(spider_res.len());
    for item in spider_res {
        let temp = FitnessAppointRes {
            appo_desc: item.appo_desc,
            show_time: item.show_time,
            test_time: item.test_time,
            test_type: item.test_type,
            class_name: item.class_name,
            status: item.status,
        };
        res.push(temp);
    }
    Ok(res.into())
}
