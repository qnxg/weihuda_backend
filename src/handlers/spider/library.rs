use axum::Extension;
use tokio::try_join;

use crate::{
    app_result::AppResult,
    entities::spider::library::{
        BorrowedRes, BorrowingRes, FinanceRes, LibraryRes, SpiderBorrowed, SpiderBorrowing,
        SpiderFinance,
    },
    utils::{jwt::parse_stu_id, request::spider_data},
};

pub async fn get_library_handler(Extension(token): Extension<String>) -> AppResult {
    let stu_id = parse_stu_id(&token)?;
    let params = [("stuid", stu_id)];
    let (borrowed, borrowing, finance): (
        Vec<SpiderBorrowed>,
        Vec<SpiderBorrowing>,
        Vec<SpiderFinance>,
    ) = try_join!(
        spider_data("/library/history_loan", &params),
        spider_data("/library/current_loan", &params),
        spider_data("/library/finance", &params),
    )?;

    let mut borrowed_res = Vec::with_capacity(borrowed.len());
    for item in borrowed {
        let temp = BorrowedRes {
            author: item.author,
            id: item.callno,
            isbn: item.isbn,
            _type: if item.logtype == "30001" {
                "借书".to_owned()
            } else {
                "还书".to_owned()
            },
            publisher: item.publisher,
            time: item.time,
            title: item.title,
            renew: item.totalRenewNum,
        };
        borrowed_res.push(temp);
    }
    let mut borrowing_res = Vec::with_capacity(borrowing.len());
    for item in borrowing {
        let temp = BorrowingRes {
            author: item.author,
            id: item.callno,
            isbn: item.isbn,
            borrowDate: item.loanDate,
            publisher: item.publisher,
            returnDate: item.returnDate,
            title: item.title,
            renew: item.totalRenewNum,
        };
        borrowing_res.push(temp);
    }
    let mut finance_res = Vec::with_capacity(finance.len());
    for item in finance {
        let temp = FinanceRes {
            barcode: item.barcode,
            title: item.bookTitle,
            bookID: item.bookrecno.to_string(),
            cost: item.cost,
            feeType: item.feetype,
            library: item.local,
            paySign: item.paySign,
            payType: item.paytype,
            time: item.regtime,
            tranID: item.tranid,
        };
        finance_res.push(temp);
    }
    let res = LibraryRes {
        borrowed: borrowed_res,
        borrowing: borrowing_res,
        finance: finance_res,
    };
    Ok(res.into())
}
