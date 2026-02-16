mod announcement;
mod auth;
mod card;
mod course;
mod electricity;
mod email;
mod exam;
mod feedback;
mod grade_rank;
mod gym;
mod jifen;
mod lab;
mod left_message;
mod netflow;
mod notice;
mod ping;
mod public_info;
mod semesters;
mod user;
mod user_setting;
mod zhihu;

use salvo::Router;

pub fn routers() -> Router {
    Router::new()
        .push(announcement::routers())
        .push(auth::routers())
        .push(card::routers())
        .push(course::routers())
        .push(electricity::routers())
        .push(email::routers())
        .push(exam::routers())
        .push(feedback::routers())
        .push(grade_rank::routers())
        .push(gym::routers())
        .push(jifen::routers())
        .push(lab::routers())
        .push(left_message::routers())
        .push(netflow::routers())
        .push(notice::routers())
        .push(ping::routers())
        .push(public_info::routers())
        .push(semesters::routers())
        .push(user_setting::routers())
        .push(user::routers())
        .push(zhihu::routers())
}
