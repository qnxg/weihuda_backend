use crate::{
    handlers::{
        gym::{
            get_gym_appoint_detail_handler, get_gym_appoint_handler,
            get_gym_grade_handler, get_gym_raw_grade_handler,
        },
        hdjw::{
            get_class_table_extra_handler, get_class_table_handler,
            get_empty_classroom_handler, get_exam_schedule_handler,
            get_grade_detail_handler, get_grade_from_ca_handler,
            get_grade_handler, get_rank_from_hdjw_handler,
        },
        lab::{
            check_lab_password_handler, get_lab_course_list_handler,
            get_lab_list_handler, get_lab_score_detail_handler,
            get_lab_score_handler, get_lab_score_structure_handler,
            get_lab_sem_info_handler, get_virtual_lab_score_handler,
        },
        library::{get_current_loan, get_finance, get_history_loan},
        netflow::{
            get_netflow_day_detail_handler, get_netflow_handler,
            get_netflow_month_detail_handler,
            get_netflow_order_handler, get_netflow_pay_info_handler,
            get_unlock_status_handler,
        },
        pt::{
            check_password_handler, get_card_history_handler,
            get_card_info_handler, get_unread_email_handler,
        },
    },
    middlewares::{
        auth_middleware, catch_panic, default_response,
        log_middleware, timeout,
    },
    utils::traffic::TrafficAnalyzerMiddleware,
};
// use salvo::logging::Logger;
use crate::handlers::{
    electricity::get_electricity_handler,
    xgxt::get_person_info_handler,
};
use salvo::{Router, Service};

pub fn create_router() -> Service {
    // Logger放在Router上不放在更外层的Service上监听不到404未匹配的请求，但是404的错误并不是很重要
    // catch_panic用于捕获panic，程序出现panic会返回500
    // timeout中间件设置最大请求时间，超时会返回503
    // auth_middleware是自定义的中间件，用于鉴权
    let base = Router::new()
        .hoop(catch_panic)
        // .hoop(Logger::new())
        .hoop(log_middleware)
        .hoop(timeout)
        .hoop(TrafficAnalyzerMiddleware::new())
        .hoop(auth_middleware);
    Service::new(
        base.push(bks())
            .push(pt())
            .push(netflow())
            .push(freeroom())
            .push(gym())
            .push(library())
            .push(xgxt())
            .push(electricity())
            .push(lab()),
    )
    // .hoop(throttle)
    .hoop(default_response)
}

// 体测系统的爬虫请求
fn gym() -> Router {
    Router::with_path("gymos")
        .push(Router::with_path("grade").get(get_gym_grade_handler))
        .push(
            Router::with_path("raw_grade")
                .get(get_gym_raw_grade_handler),
        )
        .push(
            Router::with_path("appoint")
                .get(get_gym_appoint_handler)
                .push(
                    Router::with_path("detail")
                        .get(get_gym_appoint_detail_handler),
                ),
        )
}

// 个人门户
fn pt() -> Router {
    Router::with_path("pt")
        .push(
            Router::with_path("email").get(get_unread_email_handler),
        )
        .push(
            Router::with_path("card/info").get(get_card_info_handler),
        )
        .push(
            Router::with_path("card/history")
                .get(get_card_history_handler),
        )
}

// 本科生教务系统
fn bks() -> Router {
    Router::with_path("bks")
        .push(Router::with_path("grade").get(get_grade_handler).push(
            Router::with_path("detail").get(get_grade_detail_handler),
        ))
        // 目前存在并发问题，暂时注释掉
        // .push(Router::with_path("grade/analyze").get(get_grade_rank_handler))
        // .push(Router::with_path("grade/chart").get(get_grade_chart_handler))
        .push(
            Router::with_path("rank").get(get_rank_from_hdjw_handler),
        )
        .push(
            Router::with_path("grade-from-ca")
                .get(get_grade_from_ca_handler),
        )
        .push(
            Router::with_path("classtable")
                .get(get_class_table_handler),
        )
        .push(
            Router::with_path("class-table-extra")
                .get(get_class_table_extra_handler),
        )
        .push(
            Router::with_path("exam/schedule")
                .get(get_exam_schedule_handler),
        )
        .push(Router::with_path("auth").post(check_password_handler))
}

// TODO 这里应该是要合入到 bks 里面的
// 不过考虑到，研究生应该也能够通过本科生的教务系统查看到空教室信息（毕竟这是个公共信息）
// 后期还需要进一步完善
fn freeroom() -> Router {
    Router::with_path("freeroom/list")
        .get(get_empty_classroom_handler)
}

// 校园网系统
fn netflow() -> Router {
    Router::with_path("netflow")
        .get(get_netflow_handler)
        .push(
            Router::with_path("pay_info")
                .get(get_netflow_pay_info_handler),
        )
        .push(
            Router::with_path("unlock")
                .get(get_unlock_status_handler),
        )
        .push(
            Router::with_path("month_detail")
                .get(get_netflow_month_detail_handler),
        )
        .push(
            Router::with_path("day_detail")
                .get(get_netflow_day_detail_handler),
        )
        .push(
            Router::with_path("order").get(get_netflow_order_handler),
        )
}

// 图书馆系统
fn library() -> Router {
    Router::with_path("library")
        .push(Router::with_path("history_loan").get(get_history_loan))
        .push(Router::with_path("current_loan").get(get_current_loan))
        .push(Router::with_path("finance").get(get_finance))
}

// 学工系统
fn xgxt() -> Router {
    Router::with_path("xgxt").push(
        Router::with_path("person_info").get(get_person_info_handler),
    )
}

// 宿舍电量
fn electricity() -> Router {
    Router::with_path("electricity")
        .push(Router::with_path("query").get(get_electricity_handler))
}

// 大物实验平台
fn lab() -> Router {
    Router::with_path("lab")
        .push(
            Router::with_path("list")
                .push(
                    Router::with_path("lab")
                        .get(get_lab_list_handler),
                )
                .push(
                    Router::with_path("course")
                        .get(get_lab_course_list_handler),
                ),
        )
        .push(
            Router::with_path("checkPassword")
                .get(check_lab_password_handler),
        )
        .push(
            Router::with_path("sem_info")
                .get(get_lab_sem_info_handler),
        )
        .push(
            Router::with_path("score")
                .get(get_lab_score_handler)
                .push(
                    Router::with_path("structure")
                        .get(get_lab_score_structure_handler),
                )
                .push(
                    Router::with_path("detail")
                        .get(get_lab_score_detail_handler),
                )
                .push(
                    Router::with_path("virtual")
                        .get(get_virtual_lab_score_handler),
                ),
        )
}
