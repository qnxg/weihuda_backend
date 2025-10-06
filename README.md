# Rust Full Back

将原有的中间件和后端合并，用Rust语言重写，提高性能。Web框架采用axum，异步运行时采用Tokio，数据库采用Sqlx，json和yaml的解析采用serde。

## 运行

1. 前往[Rust官网](https://www.rust-lang.org/tools/install)按指示安装好Rust工具链
2. VSCode安装rust-analyzer插件，用来提供代码提示
3. `cargo run`即可运行
4. 发布时候运行`cargo build --release`生成可执行文件，Linux客户端最好在WSL环境编译。
5. 注意：不是湖南大学校园网环境需要开启**湖南大学VPN**，这样才能连接到数据库完成sql语句的静态结构检查，否则无法成功编译。DATABATE_URL在.env文件中配置，也可以创建一个相同结构的本地数据库用来供结构静态检查。

## 项目结构

```shell
|-- Cargo.toml      // 项目依赖配置
|-- Makefile        // 暂时没有用到
|-- config          // 配置文件
|-- logs            // 日志文件
|-- migrations      // sqlx的数据库迁移文件
|-- rustfmt.toml    // rust代码格式化配置
|-- src             // 源代码
|-- .env            // 环境变量配置，编译时候sqlx会根据这个文件连接数据库进行静态结构检查   
`-- target          // 编译文件
```

## 代码结构

```shell
|-- app_error.rs    // 自定义的错误类型，统一处理，逻辑层只负责传递错误
|-- app_result.rs   // 自定义的结果类型，统一处理，逻辑层只负责传递结果
|-- config.rs       // 配置文件的解析
|-- extract         // 自定义请求参数的解析，用于与自定义错误类型匹配
|   |-- json.rs     // Json请求体参数的解析
|   `-- query.rs    // Query请求头参数的解析
|-- handler         // 逻辑处理层
|   |-- back        // 后端逻辑处理层
|   `-- spider      // 爬虫逻辑处理层
|-- main.rs         // 入口文件
|-- middleware      // 中间件
|   |-- auth.rs     // 鉴权中间件，获取用户authorization字段放入Extension中
|   |-- cors.rs     // 跨域中间件
|   |-- log.rs      // 日志中间件
|   `-- timeout.rs  // 超时中间件
|-- model           // 爬虫返回数据的结构定义，返回数据结构定义
|   |-- back        // 数据库相关
|   `-- spider      // 爬虫相关
|-- router          // 路由
|   `-- mod.rs      // 路由的注册
|-- schema          // 请求参数的结构定义
|   |-- back        // 后端相关
|   `-- spider      // 爬虫相关
`-- utility
    |-- default.rs  // 一些参数的默认值
    |-- jwt.rs      // jwt的生成和解析
    |-- request.rs  // 爬虫请求的发送
    `-- wrapper.rs  // 返回数据的包装
```
## 逻辑层代码示例(以get_course_handler为例)

这个函数闲置了，因为获取自定义课程直接在获取class_table中一起实现了，这里只是一个示例

### 1. 首先定义请求参数

在schema/back/course.rs中定义请求参数的结构，获取课程请求需要xn和xq两个Query参数

```rust
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GetCourseReq {
    pub xn: u32,
    pub xq: u32,
}
```
其中展开了Deserialize宏，使得结构体能够从请求数据反序列化解析出来，继承Debug为了方便打印出结构方便调试。

### 2. 书写逻辑层代码

在handler/back/course.rs中书写逻辑层代码。

* 首先思考函数需要哪些参数：
1. 由于需要鉴权，所以需要获取jwt请求头。
2. 由于需要获取数据库连接，所以需要获取数据库连接池。
3. 由于需要获取请求参数且为Query参数，所以需要使用Query解析请求参数。

注意：排列顺序无特别要求，但是当需要解析Json参数时候，Json必须排列在最后一个，因为解析Json会消耗掉整个Request。

* 其次书写逻辑层代码
1. 逻辑层需要获取数据库返回值或者爬虫返回值需要在model中定义返回值的结构（需要derive Deserialize的宏用来反序列化为结构体）。函数的返回值也要定义返回值的结构（需要derive Serialize的宏用来序列化结构体为json数据）。
2. 逻辑层通用逻辑为：获取mini_bind_id或stu_id，然后根据请求参数查询数据库或者爬虫获取数据，最后返回数据。
3. 返回数据用Ok(res.into())，res为实现了Serialize trait的结构体。如果需要返回自定义错误，使用Err("错误信息".into())。其他已经定义好的错误在可能出错的函数直接用?返回。

```rust
use crate::{
    app_result::AppResult,
    schema::back::course::GetCourseReq,
    utility::jwt::parse_id,
    Pool, model::back::course::CourseInfo,
    extract::{Json, Query}
};
use axum::{
    extract::State,
    Extension, 
};
use std::sync::Arc;

pub async fn get_course_handler(
    State(data): State<Arc<Pool>>,
    Query(req): Query<GetCourseReq>,
    Extension(token): Extension<String>,
) -> AppResult {
    let mini_bind_id = parse_id(&token)?;

    let res = sqlx::query_as!(
        CourseInfo,
        r#"
        SELECT id, classname, location, teachers, week, day, section FROM mini_course WHERE xn = ? AND xq = ? AND mini_bind_id = ? AND deleted_at IS NULL
        "#,
        req.xn,
        req.xq,
        mini_bind_id,
    )
    .fetch_all(&data.db)
    .await?; // the type of res is Vec<CourseInfo>

    Ok(res.into())
}
```
### 3. 将逻辑层代码注册到路由中
1. 如果有合适的归类直接放到归类里，用.route()注册。
```rust
    // 后端路由
    let course = Router::new()
        .route("/course", get(get_course_handler)) // 获取自定义课表课程
        .route("/course", post(add_course_handler)) // 添加自定义课表课程
        .route("/course", delete(delete_course_handler)); // 删除自定义课表课程
```
2. 检查所在的归类是否被放到了路由分类里，需要解析token就意味需要auth，需要访问数据库就意味着需要db
```rust
    // 按所需权限分类总结，注重api的权限划分严谨性，用到什么权限就分配什么权限
    let with_db = Router::new()
        .merge(auth)
        .merge(user_bind)
        .merge(message)
        .merge(feedback)
        .with_state(db_pool.clone());

    let with_db_auth = Router::new()
        .merge(course)
        .merge(user_unbind)
        .merge(exam_num)
        .merge(class_table)
        .layer(auth_middleware())
        .with_state(db_pool.clone());

    let with_auth = Router::new()
        .merge(grade)
        .merge(exam)
        .merge(user_info)
        .merge(netflow)
        .merge(pt)
        .merge(library)
        .layer(auth_middleware());

    let without = Router::new()
        .merge(ping)
        .merge(class_start_date)
        .merge(empty_room)
        .merge(semester_info)
        .merge(zhihu);
```
3. 最后的合并代码函数返回一般不用修改
```rust
    // 合并所有router
    Router::new()
        .merge(without)
        .merge(with_auth)
        .merge(with_db_auth)
        .merge(with_db)
        .layer(log_middleware()) // 增加日志中间件
        .layer(timeout_middleware()) // 增加超时中间件
```