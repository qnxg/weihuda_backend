# 湖南大学微生活后端

演示账号：

- 学号：202506050175
- 密码：hnuwsh2025


## 1. 运行

### 1.1 准备

1. 前往[Rust 官网](https://www.rust-lang.org/tools/install)按照官方说明安装 Rust 工具链。  
2. VS Code 安装 rust-analyzer 插件，用来提供代码提示。  
3. 数据抓取与解析功能已独立为 [`hnu_query`](https://github.com/qnxg/hnu_query) 库，作为依赖引入，无需单独运行爬虫程序。
4. 仅在调试“大物实验平台”相关功能时需要额外部署验证码识别（OCR）服务。见 [4.4](#44-captcha-验证码服务)。  
5. 中间件：MySQL、Redis 是必须依赖，必须完成配置；
如果需要调试问题反馈相关接口，则需要额外部署 RabbitMQ。由于 RabbitMQ 连接采用懒加载机制，仅在实际调用相关功能时才会建立连接，因此不使用问题反馈功能时无需配置 RabbitMQ。
详情请见 [4. 外部依赖](#4-外部依赖)

目前本地开发支持直接使用项目自带的 `docker-compose.dev.yml`，一键启动本地开发所需的中间件：

   ```bash
   docker compose -f docker-compose.dev.yml up -d
   ```

   它会部署 MySQL（3306）、Redis（6379）、RabbitMQ（5672/15672）、GreptimeDB（4000）。  
首次执行时，Docker Compose 会从 Docker 镜像仓库拉取所需的镜像。由于镜像体积较大，首次启动可能需要较长时间；如镜像拉取速度较慢，可根据实际网络环境配置镜像加速源。

### 1.2 配置

运行项目之前，需要完成相关配置。  
把 `config/config_example.toml` 复制并另存为 `config/config.toml` 按照字段提示手动填入信息。

本地开发时，通常只需配置 MySQL 和 Redis；RabbitMQ、GreptimeDB、Captcha 等组件根据实际调试需求进行配置。  
建议你将 `[server].log_level` 设为 `debug` 以方便调试（日志固定 pretty 输出到 stdout）。

项项目的 `.env` 中配置了 `sqlx` 编译期静态检查所使用的数据库连接信息，通常应与 `config.toml` 中的 MySQL 配置保持一致。  
在编译过程中，sqlx 会读取 .env 中的数据库连接信息，并连接数据库对代码中的 SQL 语句进行编译期检查。如果无法连接该数据库，相关 sqlx 编译期检查将无法完成，从而可能导致项目无法通过编译。

`config/frontend_private.pem` 为用于解密前端 RSA 加密密码的私钥。项目使用 RSA 2048 位密钥对（PKCS#8），公钥与私钥需配套使用。密钥对请按项目要求自行生成，并将私钥保存至 `config/frontend_private.pem`。  
私钥属于敏感信息，请勿提交至 Git 仓库。

_小Tip：像上面这种需要在本地修改配置文件但不需要同步到云端的，可以使用 `git update-index --skip-worktree filename` 来忽略本地更改。若pull时远程仓库修改了这个文件，Git会发现冲突并提醒你，防止冲掉本地的配置。使用`--no-skip-worktree`恢复跟踪。_

### 1.3 运行
执行 `cargo run` 即可运行。

初次运行项目，Cargo 需要下载项目所需要的各种依赖库，有可能会出现等待时间过长的情况（比如 VS Code 打开项目之后一直停留在 fetching metadata 状态），出现这种情况的话你可能需要配置 Cargo 的镜像。


## 2. 项目结构

### 2.1 概览

```shell
|-- .github              // CI 配置（PR 检查、release 构建）
|-- Cargo.toml           // 项目依赖配置
|-- config               // 配置文件
|-- docs                 // 接口文档
|-- rustfmt.toml         // Rust 代码格式化配置
|-- src                  // 源代码
|-- .env                 // 环境变量配置，编译时候sqlx会根据这个文件连接数据库进行静态结构检查
|-- example.env          // .env 的示例
|-- docker-compose.dev.yml // 用于一键部署本地 MySQL / Redis / RabbitMQ / GreptimeDB
`- Dockerfile
```

### 2.2 代码结构

```shell
|-- infra               // 后端用到的所有外部组件
|   |-- cache           // Redis 缓存
|   |-- mysql           
|   |-- captcha.rs      // 大物实验室验证码识别服务
|   |-- rabbitmq.rs     // 向 RabbitMQ 发布消息
|   `-- wechat.rs       // 微信公众号相关接口
|-- middlewares         // 中间件
|   |-- catch_panic.rs  // 捕获 handler 的 panic 并返回 500
|   |-- cors.rs         // 跨域中间件，设置跨域策略
|   |-- default.rs      // 如果各个 handler 都没有渲染 JSON 格式的响应，那么这个中间件会将响应转为 JSON 格式，确保后端的响应一定是 JSON
|   |-- timeout.rs      // 超时中间件，防止请求耗时太长
|   `-- tracing.rs      // 请求埋点中间件，记录请求/响应信息与 Trace span
|-- routers             // 后端提供的所有 HTTP 接口都定义在这里
|-- service             // 具体业务逻辑
|-- utils               // 工具函数
|   |-- crypto.rs       // 对要存放在数据库中的密码进行加密
|   |-- jwt.rs          // 生成和解析 JWT
|   |-- serde.rs        // 序列化/反序列化的工具函数
|   |-- single_flight.rs // 合并同一时刻的重复请求
|   |-- task_queue.rs   // 去重的任务队列
|   |-- time.rs         // 和时间有关的工具函数
|   `-- tracing.rs      // 埋点工具
|-- config.rs           // 配置文件的解析
|-- error.rs            // 统一的错误类型
|-- main.rs             // 入口文件
`-- observability.rs    // 日志 + OTLP Trace 上报

```

## 3. 开发

### 3.1 分层

目前整个后端分为三层：

- `router` 层负责解析、校验请求参数和鉴权
- `infra` 层调用各种外部组件
- `service` 层做具体的业务逻辑

具体来说，`router` 层只需要解析校验参数和鉴权，具体业务全部交给 `service` 层。`router` 层只能调用 `service` 层，不能直接调用 `infra` 层。数据库、Redis 这些外部组件的读写都在 `infra` 层。`service` 层可以调用其它的 `service`，也可以调各种 `infra`。  
数据的缓存和校内系统数据的解析也在 `service` 层。有时可能我们就想直接在 `router` 层调用 `infra` 层，那么我们可以将 `infra` 层的函数使用 `pub use` 来转发到 `service` 层，然后调用 `service` 层的这个函数。

另外，所有校内系统的数据都来自 [`hnu_query`](https://github.com/qnxg/hnu_query) 这个库，抓取和解析都在里面做。  
后端在 `service` 层统一用 `service::user_state` 提供的 `with_token(系统::new(学号), |token| async move { hnu_query::模块::函数(&token, ...).await })` 来调用，框架会自动处理 Token 的缓存、过期刷新和重试。

调 `service` 层和 `infra` 层时最好写成 `service::exam::get_exam_arrange`、`infra::mysql::exam_num::get_exam_num_list` 这种全路径，以规避不同层同名函数出现混淆的情况。

### 3.2 接口编写示例

假如我要增加一个查询考试安排的接口，考试安排来自教务系统（`hdjw`），对应 `hnu_query::hdjw::get_exam_schedule`函数。  
先在 `service/exam.rs` 写如下代码：

```rust
#[derive(Serialize, Debug)]
pub struct ExamArrange {
    pub id: String,  // 每个字段的注释都写好（虽然这里没写完）
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

### 3.3 测试

写完一个接口之后你需要测试，保证你的代码是正常工作的。

首先你应该在 `service` 层添加单元测试。

然后你需要使用接口测试工具直接请求你添加的接口进行测试。  
如果接口需要鉴权，那么你在请求接口的时候需要携带 `Authorization` 请求头，内容为 JWT 令牌。  
JWT 令牌的生成可以通过 `utils/jwt.rs` 中的 `test_auth` 函数获得，将其中的学号修改为自己的学号后运行该测试，即可获取 JWT Token。

### 3.4 约定

- 向数据库中添加数据时，涉及到的时间均为 UTC+8
- 使用 `xn` 和 `xq` 来表示一个学年学期。
  - `xn` 为当前学年学期的起始日期，比如 2025-2026 学年，`xn` 值为 2025
  - `xq` 为 0 表示秋季学期，为 1 表示春季学期，为 2 表示夏季学期

## 4. 外部依赖
### 4.1 MySQL / Redis

- 本地开发环境中，MySQL 默认映射至宿主机 3306 端口。  
-`docker-compose.dev.yml` 里配置了默认用户名与密码：root/root，库名 `weihuda`。
- Redis默认映射至宿主机 6379 端口。

### 4.2 可选项：RabbitMQ（问题反馈）

用户提交问题反馈时，后端先把反馈写进 MySQL，再向 RabbitMQ 的一个 Exchange 发布消息，交给消费方处理。

- 配置在 `[rabbitmq]`：`url`、`feedback_exchange`。
- 连接是懒加载的，也不会自动重连——所以RabbitMQ 重启后后端要跟着重启。

### 4.3 可选项：GreptimeDB

用于接收后端上报的 Trace 数据，以便分析请求链路和请求耗时。未配置该地址时，不会上报 Trace 数据，仅保留 stdout 日志输出。

- 配置在 `[observability]` 的 `endpoint`。本地部署填写 `http:/localhost:4000/v1/otlp`即可。
- `public.opentelemetry_traces` 表会在首次收到 Trace 数据后创建。因此，服务刚启动时查询不到该表属于正常现象。

### 4.4 可选项：Captcha 验证码服务

- 源码仓库为[captcha_service](https://github.com/qnxg/captcha_service)
- 本地部署方式如下：
```Bash
cd captcha_service
docker build -t captcha_service .
docker run -d -p 5000:5000 captcha_service
```
- 配置在 `[captcha]` 的 `captcha_url`，服务地址结尾不要添加斜杠，例如本地地址为： `http://localhost:5000`。  


