# Rust Full Back

将原有的中间件和后端合并，用 Rust 语言重写，提高性能。Web 框架采用 axum，异步运行时采用 Tokio，数据库采用 Sqlx，json 和 yaml 的解析采用 serde。

## 1. 运行

### 1.1 准备

1. 前往[Rust 官网](https://www.rust-lang.org/tools/install)按指示安装好 Rust 工具链。
2. VSCode 安装 rust-analyzer 插件，用来提供代码提示。
3. 由于后端还依赖于爬虫，所以请参考爬虫的文档将爬虫运行起来。
4. 安装 MySQL 和 Redis。爬虫也需要 Redis，前端和爬虫应该共用一个 Redis 服务。

### 1.2 配置

项目的 `config/config.toml` 是配置信息，你需要先填写好配置信息才能成功运行本项目。一般来说你需要将其中的 MySQL 和 Redis 的信息填成自己本地的。还建议你将 `log` 中的 `filter_level` 设为 `debug`，将 `with_ansi` 设为 `true` 以方便调试。

项目的 `.env` 中配置了静态检查的数据库地址，一般和 `config.toml` 中的配置是一样的。最开始 `.env` 中给出了一个测试数据库地址，这个地址需要在校园网内部才能连接。在程序编译时，rust 会自动连接 `.env` 中的数据库，然后检查程序中的 sql 代码是否正确。如果连接不上的话会无法通过编译。

测试数据库还有一个作用，就是他提供了项目所需要的表结构。你可以将测试数据库中的表结构导入到你本地的数据库中，然后就可以使用本地的数据库进行开发了。诚然，你可以直接连接测试数据库进行开发，但是这样有可能会泄漏你的个人信息。

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
|-- app_error.rs    // 自定义的错误类型，统一处理，逻辑层只负责传递错误
|-- app_result.rs   // 自定义的结果类型，统一处理，逻辑层只负责传递结果
|-- config.rs       // 配置文件的解析
|-- database.rs     // 数据库连接
|-- main.rs         // 入口文件
|-- dtos、entities  // 各种结构体的定义都在这里了，目前这里比较屎，后面会考虑重构，先凑合看看
|-- extract         // 自定义请求参数的解析，用于与自定义错误类型匹配
|   |-- json.rs     // Json请求体参数的解析
|   `-- query.rs    // Query请求头参数的解析
|-- handler         // 逻辑处理层
|   |-- back        // 仅后端就可以处理的接口
|   `-- spider      // 需要调用爬虫来处理的接口
|-- middleware      // 中间件
|   |-- auth.rs     // 鉴权中间件，获取用户authorization字段放入Extension中
|   |-- cache.rs    // 缓存中间件，缓存部分接口
|   |-- cors.rs     // 跨域中间件，允许跨域请求
|   |-- count.rs    // 计数中间件，用于统计请求成功率
|   |-- log.rs      // 日志中间件，为所有请求和响应增加日志
|   `-- timeout.rs  // 超时中间件
|-- routers         // 路由
|   `-- mod.rs      // 路由的注册
`-- utils
    |-- default.rs  // 一些参数的默认值
    |-- jwt.rs      // jwt的生成和解析
    |-- lazy_cache_cell.rs  // 实现了一个可以缓存某个函数一段时间的宏，用于定期刷新学期表
    |-- redis.rs    // redis 相关操作
    |-- request.rs  // 爬虫请求的发送
    |-- semester.rs // 学期相关
    |-- serde.rs    // 对某些类型的序列化/反序列化函数
    `-- wrapper.rs  // 统一响应
```

## 3. 开发

后端所有的接口都定义在 `routers/mod.rs` 中，接口的请求方法在代码中也能很容易看出来。具体的请求参数可以直接跳转到相应的 `handler` 函数，根据函数的参数定义就能看出来了。

由于存在 `middleware/auth.rs` 中间件的鉴权，在请求接口的时候需要携带 `Authorization` 请求头，内容为 jwt 令牌。jwt 令牌的生成可以参考 `middleware/auth.rs` 代码的 `test_auth` 函数，将里面的学号改成自己的时候跑一下这个测试就能得到。
