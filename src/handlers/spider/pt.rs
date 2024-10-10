use axum::Extension;

use crate::{
    app_result::AppResult,
    dtos::spider::pt::{GetCardHistoryReq, GetFitnessReq, GetLabGradeReq},
    entities::spider::{
        card::{
            CardHistoryRes, CardHistoryResItem, CardInfoRes, SpiderCardHistory, SpiderCardInfo,
        },
        email::SpiderEmail,
        fitness::{
            get_class_color, FitnessAppointRes, FitnessRes, FitnessResEye, FitnessResItem,
            FitnessResReport, FitnessResStudent, FitnessResTotal, SpiderFitness,
            SpiderFitnessAppoint, SpiderFitnessRaw,
        },
        lab::{LabArrangeRes, LabGradeRes, LabGradeResTotal, SpiderLabArrange, SpiderLabGrade},
    },
    extractors::Query,
    utils::{
        jwt::parse_stu_id,
        request::{spider, spider_data},
    },
};

/// 获取校园一卡通信息
pub async fn get_card_info_handler(Extension(token): Extension<String>) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("stuid", stu_id)];
    let spider_res: SpiderCardInfo = spider_data("/pt/card/info", &params).await?;

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
    let params =
        [("stuid", stu_id), ("year", req.year), ("month", req.month), ("type", req._type)];
    let spider_res: SpiderCardHistory = spider_data("/pt/card/history", &params).await?;

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
pub async fn get_email_handler(Extension(token): Extension<String>) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("stuid", stu_id)];
    let spider_res: SpiderEmail = match spider_data("/pt/email", &params).await {
        Ok(res) => res,
        Err(_) => return Ok(serde_json::Value::Null.into()),
    };

    match spider_res.unReadCount {
        Some(count) => Ok(count.into()),
        None => Ok(serde_json::Value::Null.into()),
    }
}

/// 获取实验成绩
pub async fn get_lab_grade_handler(
    Query(req): Query<GetLabGradeReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("stuid", stu_id), ("labid", req.labId)];
    let spider_res: Result<SpiderLabGrade, anyhow::Error> =
        spider_data("/lab/grade", &params).await; // 由于即使数据为空也不能返回错误，所以这里不使用?操作符

    let mut res = LabGradeRes {
        items: Vec::new(),
        total: LabGradeResTotal { cj: "0".to_string(), xs: "0".to_string() },
    };
    if let Ok(spider_res) = spider_res {
        res.total.cj = spider_res.zcj;
        res.total.xs = spider_res.zxs;
        res.items = spider_res.items;
    }
    Ok(res.into())
}

/// 获取实验安排
pub async fn get_lab_arrange_handler(Extension(token): Extension<String>) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("stuid", stu_id)];
    let spider_res: Result<Vec<SpiderLabArrange>, anyhow::Error> =
        spider_data("/lab/arrange", &params).await; // 由于即使数据为空也不能返回错误，所以这里不使用?操作符

    let mut res = Vec::new();
    if let Ok(spider_res_items) = spider_res {
        for item in spider_res_items {
            let temp = LabArrangeRes {
                classname: item.classname,
                classtype: item.classtype,
                date: item.labdate,
                time: item.labtime,
                week: item.labweek,
                name: item.labname,
                place: item.labplace,
            };
            res.push(temp);
        }
    }
    Ok(res.into())
}

/// 获取体测成绩
pub async fn get_fitness_handler(
    Query(req): Query<GetFitnessReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("stuid", stu_id), ("xn", req.xn)];
    let data: SpiderFitness = spider("/gymos/grade", &params).await?;
    let raw: SpiderFitnessRaw = spider_data("/gymos/raw_grade", &params).await?;

    let mut res = FitnessRes {
        student: FitnessResStudent { name: raw.student_name, number: raw.student_num },
        total: FitnessResTotal { score: raw.total_score, grade: raw.total_grade },
        report: FitnessResReport {
            desc: raw.report_desc,
            status: data.report_status.unwrap_or(raw.status.to_string()),
            _type: raw.report_type.to_string(),
        },
        eye: FitnessResEye {
            eyesight_right: format!("{} {}", raw.eyesight_right, raw.eyesight_right_detail),
            eyesight_left: format!("{} {}", raw.eyesight_left, raw.eyesight_left_detail),
            eye_mirror_right: format!("{} {}", raw.eye_mirror_right, raw.eye_mirror_right_detail),
            eye_mirror_left: format!("{} {}", raw.eye_mirror_left, raw.eye_mirror_left_detail),
            eye_ametropia_right: format!(
                "{} {}",
                raw.eye_ametropia_right, raw.eye_ametropia_right_detail
            ),
            eye_ametropia_left: format!(
                "{} {}",
                raw.eye_ametropia_left, raw.eye_ametropia_left_detail
            ),
        },
        items: Vec::new(), // To be filled
    };
    res.items.push(FitnessResItem {
        name: "50m".to_string(),
        class: data.data.short_run_class.unwrap_or(get_class_color(&raw.short_run_grade)),
        score: data.data.short_run_score.unwrap_or(raw.short_run + "秒"),
        rank: raw.short_run_grade,
        grade: raw.short_run_score,
    });
    res.items.push(FitnessResItem {
        name: "BMI".to_string(),
        class: data.data.bmi_class.unwrap_or(get_class_color(&raw.bmi_grade)),
        score: data
            .data
            .bmi_score
            .unwrap_or(format!("{}厘米/{}千克", raw.height, raw.weight)),
        rank: raw.bmi_grade,
        grade: raw.bmi_score,
    });
    res.items.push(FitnessResItem {
        name: "跳远".to_string(),
        class: data.data.jump_class.unwrap_or(get_class_color(&raw.jump_grade)),
        score: data.data.jump_score.unwrap_or(raw.jump + "厘米"),
        rank: raw.jump_grade,
        grade: raw.jump_score,
    });
    res.items.push(FitnessResItem {
        name: "引体向上/仰卧起坐".to_string(),
        class: data
            .data
            .pull_and_sit_class
            .unwrap_or(get_class_color(&raw.pull_and_sit_grade)),
        score: data.data.pull_and_sit_score.unwrap_or(raw.pull_and_sit.to_string()),
        rank: raw.pull_and_sit_grade,
        grade: raw.pull_and_sit_score + raw.extra_score_pull_or_sit_up,
    });
    res.items.push(FitnessResItem {
        name: "长跑".to_string(),
        class: data.data.run_class.unwrap_or(get_class_color(&raw.run_grade)),
        score: data.data.run_score.unwrap_or({
            let total_seconds: u32 = raw.run.parse().unwrap_or(0);
            let minutes = total_seconds / 60;
            let seconds = total_seconds - minutes * 60;
            if seconds != 0 {
                format!("{}'{}\"", minutes, seconds)
            } else {
                format!("{}'", minutes)
            }
        }),
        rank: raw.run_grade,
        grade: raw.run_score + raw.extra_score_run,
    });
    res.items.push(FitnessResItem {
        name: "坐位体前屈".to_string(),
        class: data
            .data
            .sit_and_reach_class
            .unwrap_or(get_class_color(&raw.sit_and_reach_grade)),
        score: data.data.sit_and_reach_score.unwrap_or(raw.sit_and_reach + "厘米"),
        rank: raw.sit_and_reach_grade,
        grade: raw.sit_and_reach_score,
    });
    res.items.push(FitnessResItem {
        name: "肺活量".to_string(),
        class: data.data.vc_class.unwrap_or(get_class_color(&raw.vc_grade)),
        score: data.data.vc_score.unwrap_or(raw.vc.to_string() + "毫升"),
        rank: raw.vc_grade,
        grade: raw.vc_score,
    });
    Ok(res.into())
}

pub async fn get_fitness_appoint_handler(Extension(token): Extension<String>) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("stuid", stu_id)];
    let spider_res: Vec<SpiderFitnessAppoint> = spider_data("/gymos/appoint", &params).await?;

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
