# 湖南大学微生活后端

## 1. 运行

### 1.1 准备

1. 前往[Rust 官网](https://www.rust-lang.org/tools/install)按指示安装好 Rust 工具链。
2. VSCode 安装 rust-analyzer 插件，用来提供代码提示。
3. 由于后端还依赖于爬虫，所以请参考爬虫的文档将爬虫运行起来。
4. 安装 MySQL 和 Redis。爬虫也需要 Redis，前端和爬虫应该共用一个 Redis 服务。
5. 如果你需要调试增加问题反馈的接口，那么还需要安装 RabbitMQ（由于与 RabbitMQ 的连接实例是懒加载的，所以你不去真的使用 RabbitMQ 那么就不用配置 RabbitMQ，目前代码中只有问题反馈用到了 RabbitMQ），设置一个 fanout 类型的 exchange 即可。

### 1.2 配置

项目的 `config/config.toml` 是配置信息，你需要先填写好配置信息才能成功运行本项目。一般来说你需要将其中的 MySQL 和 Redis 的信息填成自己本地的。还建议你将 `log` 中的 `filter_level` 设为 `debug`，将 `with_ansi` 设为 `true` 以方便调试。

项目的 `.env` 中配置了静态检查的数据库地址，一般和 `config.toml` 中的配置是一样的。最开始 `.env` 中给出了一个测试数据库地址。在程序编译时，rust 会自动连接 `.env` 中的数据库，然后检查程序中的 sql 代码是否正确。如果连接不上的话会无法通过编译。

测试数据库还有一个作用，就是他提供了项目所需要的表结构。你可以将测试数据库中的表结构导入到你本地的数据库中，然后就可以使用本地的数据库进行开发了。诚然，你可以直接连接测试数据库进行开发，但是这样有可能会泄漏你的个人信息。

同时，由于每次 Rust 执行代码检查时都会去连接这个测试数据库，我们建议你把测试数据库在本地复制一份，然后把 `.env` 中的数据库信息改为自己本地数据库的信息，这样 Rust 的代码检查会快一点。

### 1.3 其他

执行 `cargo run` 即可运行。

初次运行项目，cargo 需要下载项目所需要的各种依赖库，有可能会出现等待时间过长的情况（比如 vscode 打开项目之后一直在 fetching metadata），出现这种问题的话你可能需要配置 cargo 的镜像。

请在提交代码前注意不要把配置文件，如 `config/config.toml`、`.env` 提交上去。

## 2. 项目结构

### 2.1 概览

```shell
|-- .gitea          // gitea 的 action 配置
|-- Cargo.toml      // 项目依赖配置
|-- config          // 配置文件
|-- logs            // 日志文件
|-- rustfmt.toml    // rust代码格式化配置
|-- src             // 源代码
|-- .env            // 环境变量配置，编译时候sqlx会根据这个文件连接数据库进行静态结构检查
`-- target          // 编译文件
```

### 2.2 代码结构

```shell
|-- infra           // 后端用到的所有外部组件
|   |-- mysql
|   |-- spider
|   |-- rabbitmq.rs
|   |-- redis.rs
|   |-- verify.rs   // 个人门户的密码验证服务
|   `-- wechat.rs   // 微信公众号相关接口
|-- middlewares     // 中间件
|   |-- cache.rs    // 缓存中间件，对某些路由进行缓存
|   |-- cors.rs     // 跨域中间件，设置跨域策略
|   |-- count.rs    // 计数中间件，统计请求的成功数和失败数
|   |-- default.rs  // 如果各个 handler 都没有渲染 json 格式的响应，那么这个中间件会将响应转为 json 格式，确保后端的响应一定是 json
|   `-- timeout.rs  // 超时中间件，防止一个请求的耗时太长
|-- routers         // 后端提供的所有 http 接口都定义在这里
|-- service         // 存放具体的业务逻辑
|-- utils
|   |-- crypto.rs   // 对要存放在数据库中的密码进行加密
|   |-- jwt.rs      // 生成和解析 jwt
|   |-- serde.rs    // 序列化/反序列化的一些工具函数
|   `-- time.rs     // 和时间有关的一些工具函数
|-- config.rs       // 配置文件的解析
|-- main.rs         // 入口文件
`-- result.rs       // 定义了统一的错误类型
```

## 3. 开发

### 3.1 分层

目前整个后端分成三层：

- `router` 层负责解析和校验请求的参数、鉴权
- `infra` 层来调用各种外部组件
- `service` 层负责具体的业务逻辑

具体来说，`router` 层只需要负责解析和校验请求的参数和鉴权，具体的业务逻辑全部交给 `service` 层。`router` 层只能调用 `service` 层而不能直接调用 `infra` 层。有关外部组件的行为，比如数据库的操作，或者调用爬虫之类的，全部在 `infra` 层做。`service` 层可以调用其他的 `service`，或是调用各种 `infra`，数据的缓存和爬虫数据的解析也在 `service` 层做。有时可能我们就想直接在 `router` 层调用 `infra` 层，那么我们可以将 `infra` 层的函数使用 `pub use` 来转发到 `service` 层，然后调用 `service` 层的这个函数。

调用 `service` 层和 `infra` 层时我们最好写成类似 `service::exam::get_exam_arrange` 和 `infra::spider::hdjw::get_exam_arrange` 这种形式，防止混淆不同层相同名称的函数。

### 3.2 接口编写示例

假如我要增加一个查询考试安排的接口，那么首先需要在爬虫中添加相应的接口，然后在后端的 `infra/spider` 中添加对接爬虫的代码。考试安排应该是来自于 `hdjw` 的，因此在 `infra/spider/hdjw.rs` 中加入如下代码：

```rust
#[derive(Deserialize, Debug)]
pub struct SpiderExamArrangeItem { // <----- 根据爬虫返回的数据格式写结构体类型。结构体名称最好以 Spider 开头
    pub kch: String,         // 课程代码 // <----- 每个字段的注释都要写好
    pub kskcmc: String,      // 课程名称
    pub ksxq: String,        // 考试校区
    pub js_mc: String,       // 考试的教室
    pub kssj: String,        // 考试时间（已经是一个时间区间了）
    pub zwh: Option<String>, // 座位号
}
pub async fn get_exam_arrange(
    stu_id: &str,
    xn: u32,
    xq: u32,
) -> AppResult<Vec<SpiderExamArrangeItem>> {
    let params = [ // <------ 按照这种形式填写调用爬虫所需参数
        ("xn", xn.to_string()),
        ("xq", xq.to_string()),
        ("stuid", stu_id.to_string()),
    ];
    let spider_res: Vec<SpiderExamArrangeItem> =
        spider_data("/bks/exam/schedule", &params).await?; // <------ 使用 spider_data 函数调用爬虫
    Ok(spider_res)  // <------ 不需要再多做太多的数据解析，直接把爬虫响应的数据返回就好了，具体的数据解析交给 service 层
}
```

然后编写 `service` 层代码，在 `service/exam.rs` 中添加如下代码：

```rust
#[derive(Serialize, Debug)]
pub struct ExamArrange {
    pub id: String,  // <----- 每个字段的注释都写好（虽然这里只写了一部分（））
    pub name: String,
    pub place: String,
    pub date: String, // 考试日期，格式为 "YYYY-MM-DD"
    pub time: String, // 考试的时间段，例如：14:00~16:00
    pub seat: String,
}
pub async fn get_exam_arrange(
    stu_id: &str,
    xn: u32,
    xq: u32,
) -> AppResult<Vec<ExamArrange>> { // <----- service 层主要是将爬虫的 SpiderExamArrange 中的信息解析为我们自己的 ExamArrange 类型
    let spider_res =
        infra::spider::hdjw::get_exam_arrange(stu_id, xn, xq).await?; // <----- 调用爬虫
    // <----- 下面的代码就是数据的解析了
    let mut res = Vec::new();
    for item in spider_res {
        let date_time_parts: Vec<&str> =
            item.kssj.split(' ').collect();
        if date_time_parts.len() != 2 {
            return Err(
                anyhow!("解析考试时间失败：{}", item.kssj).into()
            );
        }
        let temp = ExamArrange {
            id: item.kch,
            name: item.kskcmc,
            place: format!("{} {}", item.js_mc, item.ksxq),
            date: date_time_parts[0].to_string(),
            time: date_time_parts[1].to_string(),
            seat: item.zwh.unwrap_or_else(|| "无".to_string()),
        };
        res.push(temp);
    }
    res.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(res)
}
```

然后在 `router` 层提供对应的接口，在 `routers/exam.rs` 中添加下面的代码：

```rust
#[derive(Deserialize, Debug)]
struct GetExamArrangeReq { // <----- 我们建议把需要的参数全部放到一个结构体中
                           //        结构体的命名格式为 请求方式+涉及到的实体名称+Req
                           //        用于参数解析的结构体不要 pub
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub xn: Option<u32>,  // <----- 建议请求参数中的可选参数都加上上面这两个宏，这样对于空字符串就会识别成 None
    #[serde(default)]
    #[serde(deserialize_with = "empty_string_as_none")]
    pub xq: Option<u32>,
}
#[handler] // <----- salvo 的 handler 要加上这个
async fn get_exam_arrange(req: &mut Request) -> RouterResult { // <----- 不要 pub，一般要加上 `req: &mut Request`，接口的返回类型一定是 RouterResult
    let (_, stu_id) = utils::jwt::auth(req)?; // <----- 使用 `utils::jwt::auth` 来鉴权，鉴权失败会自动抛出错误
    let GetExamArrangeReq { xn, xq } = req.parse_queries()?; // <----- 参数的解析建议使用解构语法
    // <----- router 层可以对参数进行加工，比如这里就给没传递学年学期的参数变为当前的学年学期
    let (current_xn, current_xq) =
        service::semester::get_now_xnxq().await?;
    let xn = xn.unwrap_or(current_xn);
    let xq = xq.unwrap_or(current_xq);
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

然后在 `routers/mod.rs` 中把 `routers/exam.rs` 提供的所有接口注册进去：

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

然后你需要使用接口测试工具直接请求你添加的接口进行测试。如果接口需要鉴权，那么你在请求接口的时候需要携带 `Authorization` 请求头，内容为 jwt 令牌。jwt 令牌的生成可以通过 `utils/jwt.rs` 中的 `test_auth` 函数获得，将里面的学号改成自己的然后跑一下这个测试就能得到。
