use crate::{
    handlers::{
        back::{
            auth::{
                get_auth_handler, get_auth_qrcode_handler, get_auth_qrcode_info_handler,
                get_auth_qrcode_status_handler, put_auth_qrcode_status_handler,
            },
            config::get_config_handler,
            course::{add_course_handler, delete_course_handler},
            exam_num::{add_exam_num_handler, delete_exam_num_handler, get_exam_num_handler},
            feedback::{add_feedback_handler, get_feedback_handler, update_feedback_handler},
            message::get_message_handler,
            notice::{get_notice_handler, put_notice_by_id_handler},
            ping::health_checker_handler,
            record::{
                get_record_goods_handler, get_record_handler, get_record_rules_handler,
                get_record_total_handler, get_webview_read_handler, post_goods_handler,
                post_record_handler,
            },
            user::{bind_user_handler, unbind_user_handler},
            zhihu::{
                delete_zhihu_handler, get_zhihu_by_id_handler, get_zhihu_page_handler,
                post_zhihu_handler, put_zhihu_handler,
            },
        },
        spider::{
            hdjw::{
                get_class_start_date_handler, get_class_table_handler,
                get_computer_exam_arrange_handler, get_empty_room_handler,
                get_exam_arrange_handler, get_grade_chart_handler, get_grade_handler,
                get_grade_rank_handler, get_raw_grade_handler,
            },
            info::{get_semester_info_handler, get_user_info_handler},
            library::get_library_handler,
            netflow::{
                get_netflow_day_detail_handler, get_netflow_handler,
                get_netflow_month_detail_handler, get_netflow_order_handler,
            },
            pt::{
                get_card_history_handler, get_card_info_handler, get_email_handler,
                get_fitness_appoint_handler, get_fitness_handler, get_lab_arrange_handler,
                get_lab_grade_handler,
            },
        },
        // test::{test_naive_datetime_parsing, test_option_naive_datetime_parsing},
    },
    middlewares::{
        auth::auth_middleware,
        count::{count_middleware, Count},
        log::log_middleware,
        timeout::timeout_middleware,
    },
    DbPool,
};
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::{atomic::AtomicUsize, Arc};
use tokio::sync::RwLock;

pub fn create_router(db_pool: Arc<DbPool>) -> Router {
    let ping = Router::new().route("/ping", get(health_checker_handler));

    // 后端路由
    let course = Router::new()
        // .route("/course", get(get_course_handler)) // 获取自定义课表课程，在获取课表中合并，不在单独提供接口
        .route("/course", post(add_course_handler)) // 添加自定义课表课程
        .route("/course", delete(delete_course_handler)); // 删除自定义课表课程

    let exam_num = Router::new()
        .route("/exam-num", get(get_exam_num_handler)) // 获取考号预存
        .route("/exam-num", post(add_exam_num_handler)) // 添加考号预存
        .route("/exam-num", delete(delete_exam_num_handler)); // 删除考号预存
                                                              // .route("/exam-num", put(update_exam_num_handler))

    let auth = Router::new().route("/token", get(get_auth_handler)); // 用code换取token

    let message = Router::new().route("/message", get(get_message_handler)); // 获取消息

    let user_bind = Router::new().route("/bind", post(bind_user_handler)); // 绑定用户

    let user_unbind = Router::new().route("/unbind", post(unbind_user_handler)); // 解绑用户

    let feedback = Router::new()
        .route("/feedback/no-auth", get(get_feedback_handler))
        .route("/feedback", post(add_feedback_handler))
        .route("/feedback", put(update_feedback_handler)); // 反馈先简单地归类为无需权限

    // 教务系统路由 hdjw
    let class_table = Router::new().route("/hdjw/class-table", get(get_class_table_handler)); // 获取课表

    let class_start_date =
        Router::new().route("/hdjw/class-start-date", get(get_class_start_date_handler)); // 获取学期开课时间

    let grade = Router::new()
        .route("/hdjw/grade", get(get_grade_handler)) // 获取成绩
        .route("/hdjw/grade-rank", get(get_grade_rank_handler)) // 获取成绩排名
        .route("/hdjw/raw-grade", get(get_raw_grade_handler)) // 获取项目成绩
        .route("/hdjw/chart", get(get_grade_chart_handler)); // 获取成绩趋势

    let exam = Router::new()
        .route("/hdjw/exam-arrange", get(get_exam_arrange_handler)) // 获取考试安排
        .route("/hdjw/computer-exam", get(get_computer_exam_arrange_handler)); // 获取机考安排

    let empty_room = Router::new().route("/hdjw/empty-room", get(get_empty_room_handler)); // 获取空教室

    // 获取信息 info
    let semester_info = Router::new().route("/info/smester", get(get_semester_info_handler)); // 获取学期信息
    let user_info = Router::new().route("/info/user", get(get_user_info_handler)); // 获取用户信息

    // 校园网 netflow
    let netflow = Router::new()
        .route("/netflow/order", get(get_netflow_order_handler))
        .route("/netflow/month-detail", get(get_netflow_month_detail_handler))
        .route("/netflow/day-detail", get(get_netflow_day_detail_handler))
        .route("/netflow", get(get_netflow_handler)); // 获取校园网流量订单

    // 知湖 zhihu
    let zhihu = Router::new()
        .route("/zhihu", get(get_zhihu_page_handler))
        .route("/zhihu/:id", get(get_zhihu_by_id_handler))
        .route("/zhihu/:id", put(put_zhihu_handler))
        .route("/zhihu/:id", delete(delete_zhihu_handler))
        .route("/zhihu", post(post_zhihu_handler)); // 获取知湖列表

    // 个人门户 pt
    let pt = Router::new()
        .route("/pt/card-info", get(get_card_info_handler)) // 获取一卡通信息
        .route("/pt/email", get(get_email_handler))
        .route("/pt/card-history", get(get_card_history_handler)) // 获取一卡通消费历史
        .route("/pt/lab-arrange", get(get_lab_arrange_handler)) // 获取实验安排
        .route("/pt/lab-grade", get(get_lab_grade_handler)) // 获取未读邮件数量
        .route("/pt/fitness-appoint", get(get_fitness_appoint_handler)) // 获取体测预约
        .route("/pt/fitness", get(get_fitness_handler)); // 获取体测信息

    // 图书馆 library
    let library = Router::new().route("/library", get(get_library_handler)); // 获取图书馆信息

    // 积分
    let record = Router::new()
        .route("/jifen/total", get(get_record_total_handler))
        .route("/jifen/record", get(get_record_handler))
        .route("/jifen/goods", get(get_record_goods_handler))
        .route("/jifen/rules", get(get_record_rules_handler))
        .route("/jifen", post(post_record_handler))
        .route("/jifen/goods-record", get(get_record_handler))
        .route("/jifen/goods-record", post(post_goods_handler))
        .route("/jifen/webview-read", get(get_webview_read_handler));

    // 配置
    let config = Router::new().route("/config", get(get_config_handler));

    // 通知
    let notice = Router::new()
        .route("/notice", get(get_notice_handler))
        .route("/notice/:id", put(put_notice_by_id_handler));

    // 二维码
    let qr = Router::new()
        .route("/auth-qrcode", get(get_auth_qrcode_handler))
        .route("/auth-qrcode/status/:code", get(get_auth_qrcode_status_handler))
        .route("/auth-qrcode/info/:code", get(get_auth_qrcode_info_handler));
    let qr_auth =
        Router::new().route("/auth-qrcode/status/:code", put(put_auth_qrcode_status_handler)); // put请求需要获取jwt，所以需要加入到with_auth的路由组

    // Test 用来开发测试的接口路由
    // let test = Router::new().route("/test", post(test_option_naive_datetime_parsing));

    // 按所需权限分类总结，注重api的权限划分严谨性，用到什么权限就分配什么权限
    let with_db = Router::new()
        .merge(auth)
        .merge(user_bind)
        .merge(message)
        .merge(feedback)
        .merge(config)
        .with_state(db_pool.clone());

    let with_db_auth = Router::new()
        .merge(course)
        .merge(user_unbind)
        .merge(exam_num)
        .merge(class_table)
        .merge(zhihu)
        .merge(record)
        .merge(notice)
        .layer(auth_middleware())
        .with_state(db_pool.clone());

    let with_auth = Router::new()
        .merge(grade)
        .merge(exam)
        .merge(user_info)
        .merge(netflow)
        .merge(pt)
        .merge(library)
        .merge(qr_auth)
        .layer(auth_middleware());

    let without = Router::new()
        // .merge(test)
        .merge(ping)
        .merge(class_start_date)
        .merge(empty_room)
        .merge(semester_info)
        .merge(qr);

    // 计数的中间件
    let count_inner = Count {
        count: AtomicUsize::new(0),
        err_count: AtomicUsize::new(0),
        last_update: RwLock::new(chrono::Local::now().naive_local().date()),
    };
    let count = Arc::new(count_inner);

    // 合并所有router
    Router::new()
        .merge(without)
        .merge(with_auth)
        .merge(with_db_auth)
        .merge(with_db)
        .layer(log_middleware()) // 增加日志中间件
        .layer(timeout_middleware()) // 增加超时中间件
        .layer(analytics::Analytics::new("8a019718-bd48-4725-966b-c95af1cd316b".to_owned())) // monitor中间件测试中
        .layer(axum::middleware::from_fn_with_state(count, count_middleware)) // 启用计数中间件
}
