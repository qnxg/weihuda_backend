# 贡献指南

欢迎为湖南大学微生活后端贡献代码，本文档将为您说明项目结构、开发流程与贡献规范。

## 项目结构

### 概览

```shell
|-- .github              // CI 配置（PR 检查、release 构建）
|-- Cargo.toml           // 项目依赖配置
|-- config               // 配置文件
|-- docs                 // 接口文档
|-- rustfmt.toml         // Rust 代码格式化配置
|-- src                  // 源代码
|-- sql                  // 用于初始化 MySQL 表结构的 SQL 文件
|-- .env                 // 环境变量配置，编译时候 sqlx 会根据这个文件连接数据库进行静态结构检查
|-- example.env          // .env 的示例
|-- docker-compose.dev.yml // 用于一键部署本地开发所需的中间件
`- Dockerfile
```

### 代码结构

```shell
|-- infra                // 后端用到的所有外部组件
|   |-- cache            // Redis 缓存
|   |-- mysql            // MySQL 数据库访问
|   |-- captcha.rs       // 大物实验室验证码识别服务
|   |-- rabbitmq.rs      // 向 RabbitMQ 发布消息
|   `-- wechat.rs        // 微信公众号相关接口
|-- middlewares          // 中间件
|   |-- catch_panic.rs   // 捕获 handler 的 panic 并返回 500
|   |-- cors.rs          // 跨域中间件，设置跨域策略
|   |-- default.rs       // 如果各个 handler 都没有渲染 JSON 格式的响应，那么这个中间件会将响应转为 JSON 格式，确保后端的响应一定是 JSON
|   |-- timeout.rs       // 超时中间件，防止请求耗时太长
|   `-- tracing.rs       // 请求埋点中间件，记录请求/响应信息与 Trace span
|-- routers              // 后端提供的所有 HTTP 接口都定义在这里
|-- service              // 具体业务逻辑
|-- utils                // 工具函数
|   |-- crypto.rs        // 对要存放在数据库中的密码进行加密
|   |-- jwt.rs           // 生成和解析 JWT
|   |-- serde.rs         // 序列化/反序列化的工具函数
|   |-- single_flight.rs // 合并同一时刻的重复请求
|   |-- task_queue.rs    // 去重的任务队列
|   |-- time.rs          // 和时间有关的工具函数
|   `-- tracing.rs       // 埋点工具
|-- config.rs            // 配置文件的解析
|-- error.rs             // 统一的错误类型
|-- main.rs              // 入口文件
`-- observability.rs     // 日志 + OTLP Trace 上报
```

## 开发

### 分层

目前整个后端分为三层：

- `router` 层负责解析、校验请求参数和鉴权
- `infra` 层调用各种外部组件
- `service` 层做具体的业务逻辑

具体来说，`router` 层只需要解析校验参数和鉴权，具体业务全部交给 `service` 层。  
`router` 层只能调用 `service` 层，不能直接调用 `infra` 层。数据库、Redis 这些外部组件的读写都在 `infra` 层。  
`service` 层可以调用其它的 `service`，也可以调各种 `infra`。数据的缓存和校内系统数据的解析也在 `service` 层。  
有时可能我们就想直接在 `router` 层调用 `infra` 层，那么我们可以将 `infra` 层的函数使用 `pub use` 来转发到 `service` 层，然后调用 `service` 层的这个函数。  


调 `service` 层和 `infra` 层时注意必须写成 `service::exam::get_exam_arrange`、`infra::mysql::exam_num::get_exam_num_list` 这种全路径，以规避不同层同名函数出现混淆的情况。

### 接口编写示例

若您需要新增接口，请依照如下流程：

1. 在 service 层实现业务逻辑
2. 在 routers 层解析请求参数并鉴权
3. 将接口加入当前模块的 routers()
4. 在 routers/mod.rs 注册模块
5. 编写单元测试并使用接口测试工具验证

以下是详细过程：

假如我要增加一个查询考试安排的接口，考试安排来自教务系统（`hdjw`），对应 `hnu_query::hdjw::get_exam_schedule` 函数。  
先在 `service/exam.rs` 写如下代码：

```rust
#[derive(Serialize, Debug)]
pub struct ExamArrange {
    pub id: String,   // 每个字段的注释都写好（虽然这里没写完）
    pub name: String,
    pub place: String,
    pub date: String, // 考试日期，格式 "YYYY-MM-DD"
    pub time: String, // 考试时间段，例如 14:00~16:00
    pub seat: String,
}
pub async fn get_exam_arrange(
    stu_id: &str,
    xn: u16,  
    xq: u8,  
) -> AppResult<Vec<ExamArrange>> { // <----- service 层主要把 hnu_query 返回的数据解析为我们自己的 ExamArrange 类型
    // 使用 with_token 自动管理教务系统令牌
    let items = with_token(Hdjw::new(stu_id), |token| async move {
        hnu_query::hdjw::get_exam_schedule(&token, xn, xq).await
    })
    .await?; // <----- 调用 hnu_query 抓取数据
    let mut res = Vec::new(); // <-----对数据进行解析 
    for item in items {
        let temp = ExamArrange {
            id: item.course_id,
            name: item.course_name,
            place: match (item.area, item.classroom) {
                (Some(area), Some(classroom)) => {
                    format!("{} {}", area, classroom)
                }
                _ => "未知".to_string(),
            },
            date: item
                .date
                .map(|date| date.format("%Y-%m-%d").to_string())
                .unwrap_or("未知".to_string()),
            time: item.time.unwrap_or("未知".to_string()),
            seat: item.seat.unwrap_or_else(|| "无".to_string()),
        };
        res.push(temp);
    }
    res.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(res)
}
```

然后在 `router` 层提供对应的接口，在 `routers/exam.rs` 中添加下面的代码：

```rust
#[handler] // <----- salvo 的 handler 要加这个
async fn get_exam_arrange(req: &mut Request) -> RouterResult { // <----- 不要 pub，一般带 `req: &mut Request`，返回类型一定是 RouterResult
    #[derive(Deserialize, Debug, Extractible)]
    #[salvo(extract(default_source(from = "query")))]
    struct GetExamArrangeReq { // <----- 我们建议把需要的参数全部放到一个结构体中。结构体的命名格式为 请求方式+涉及到的实体名称+Req。同时使用宏来指定参数从哪里解析出来。
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub xn: Option<u32>, // <----- 建议请求参数中的可选参数都加上上面这两个宏，这样对于空字符串就会识别成 None
        #[serde(default)]
        #[serde(deserialize_with = "empty_string_as_none")]
        pub xq: Option<u32>,
    }
    let GetExamArrangeReq { xn, xq } = req.extract().await.parse_error()?;  // <----- 参数的解析建议使用解构语法。使用 extract 来解析参数。
    let stu_id = utils::jwt::auth(req)?; // <----- 使用 utils::jwt::auth 鉴权，鉴权失败会自动抛出错误
    // <----- router 层可以对请求参数进行预处理。例如，当请求未提供学年或学期时，此处将其设置为当前学年学期。
    let (current_xn, current_xq) =
        service::semester::get_now_xnxq().await?;
    let xn = xn.unwrap_or(current_xn) as u16; // <----- 请求参数是 u32，转成 service 需要的 u16/u8
    let xq = xq.unwrap_or(current_xq) as u8;
    let res =
        service::exam::get_exam_arrange(&stu_id, xn, xq).await?;
    Ok(res.into())
}
```

`routers` 中除了 `mod.rs` 外的每个文件都只 `pub` 一个 `routers` 函数，表示当前文件提供的接口。我们再把刚才写的接口添加到 `routers` 函数中，在 `routers/exam.rs` 中添加如下代码：

```rust
pub fn routers() -> Router {
    Router::new()
        .push(
            Router::with_path("hdjw/exam-arrange")
                .get(get_exam_arrange),
        )
}
```

最后在 `routers/mod.rs` 里注册接口：

```rust
pub fn routers() -> Router {
    Router::new()
        // ........
        .push(email::routers())
        .push(exam::routers())  // <----- 添加这一行
        .push(feedback::routers())
        // ........
}
```

### 测试

写完一个接口之后你需要测试，保证你的代码是正常工作的。

首先你应该在 `service` 层添加单元测试。

然后你需要使用接口测试工具直接请求你添加的接口进行测试。  
如果接口需要鉴权，那么你在请求接口的时候需要携带 `Authorization` 请求头，内容为 JWT 令牌。  
JWT 令牌的生成可以通过 `utils/jwt.rs` 中的 `test_auth` 函数获得，将其中的学号修改为自己的学号后运行该测试，即可获取 JWT Token。

### 约定

- 向数据库中添加数据时，涉及到的时间均为 UTC+8
- 使用 `xn` 和 `xq` 来表示一个学年学期。
  - `xn` 为当前学年学期的起始日期，比如 2025-2026 学年，`xn` 值为 2025
  - `xq` 为 0 表示秋季学期，为 1 表示春季学期，为 2 表示夏季学期

## 分支规范

- 主分支为 `master`，所有改动通过 Pull Request 合并。
- 开发新功能或修复问题时，请基于 `master` 新建分支，完成后提交 PR 合并回 `master`。

_小Tip：如果你需要在本地修改配置文件但不需要同步到云端的，可以使用 `git update-index --skip-worktree filename` 来忽略本地更改。若pull时远程仓库修改了这个文件，Git会发现冲突并提醒你，防止冲掉本地的配置。使用`--no-skip-worktree`恢复跟踪。_

## 提交规范

提交信息遵循 [Conventional Commits](https://www.conventionalcommits.org/zh-hans/) 规范，格式为 `<type>(<scope>): <subject>`。描述请使用中文。



## CI 检查

每个 PR 会触发 [PR Check](../.github/workflows/pr-check.yaml)，包含以下检查，请确保本地通过后再提交：

```bash
cargo clippy --bins --tests --examples -- -D warnings  # 代码检查，警告视为错误
cargo fmt --check                                      # 代码格式检查
cargo doc --document-private-items --no-deps           # 文档注释检查
```

提交前建议在本地执行：

```bash
cargo fmt
cargo clippy --bins --tests --examples -- -D warnings
cargo test
```

## Issue 报告规范

反馈 Bug 或提出建议时，请前往 [Github Issue](https://github.com/qnxg/weihuda_backend/issues) 新建 Issue，并尽量包含以下信息：

- 问题描述与复现步骤
- 相关的日志或报错信息
- 运行环境（操作系统、Rust 版本等）

也可以加入 [湖大微生活用户交流群](https://qm.qq.com/q/BA6FQbxXBS)（QQ 群号 1051502207）获得更快的答复。
