# 部署与运行

演示账号（用于快速验证接口）：

- 学号：202506050175
- 密码：hnuwsh2025

## 环境准备

1. 前往 [Rust 官网](https://www.rust-lang.org/tools/install) 按照官方说明安装 Rust 工具链。
2. VS Code 安装 rust-analyzer 插件，用来提供代码提示。
3. 外部依赖： 

目前本地开发支持直接使用项目自带的 `docker-compose.dev.yml`，一键启动本地开发所需的中间件：

```bash
docker compose -f docker-compose.dev.yml up -d
```

它会部署:
| 服务         | 端口               |
| ---------- | ---------------- |
| MySQL      | `3306`           |
| Redis      | `6379`           |
| RabbitMQ   | `5672` / `15672` |
| GreptimeDB | `4000`           |

首次执行时，Docker Compose 会从 Docker 镜像仓库拉取所需的镜像。由于镜像体积较大，首次启动可能需要较长时间；如镜像拉取速度较慢，可根据实际网络环境配置镜像加速源。

您也可以依照以下介绍按照业务需求自行选择需要部署的中间件

### 外部依赖

> **注意：** 使用 Docker Compose 后，无需重复启动 MySQL、Redis、RabbitMQ 和 GreptimeDB，也无需手动创建下文的 RabbitMQ 交换机、队列及绑定。Captcha 需要另行部署，且后端配置仍需按“配置”一节填写。

#### MySQL / Redis（必选）

- 本地开发环境中，MySQL 默认映射至宿主机 3306 端口。
- `docker-compose.dev.yml` 里配置了默认用户名与密码：root/root，库名 `weihuda`。
- Redis 默认映射至宿主机 6379 端口。
- 使用 Compose 时，MySQL 会在首次初始化空数据卷时自动执行 `docker/mysql/init.sql`。如果自行部署 MySQL，需要先创建 `weihuda` 数据库，再使用 MySQL 客户端导入 `docker/mysql/init.sql` 中的表结构；只有空数据库无法完成 SQLx 编译期检查。已有数据卷不会在每次启动时重新执行初始化脚本。

#### RabbitMQ（问题反馈，可选）

用户提交问题反馈时，后端先把反馈写进 MySQL，再向 RabbitMQ 的一个 Exchange 发布消息，交给消费方处理。

- 本地部署方式如下：

```bash
docker run -d --name rabbitmq -p 127.0.0.1:5672:5672 -p 127.0.0.1:15672:15672 -e RABBITMQ_SERVER_ADDITIONAL_ERL_ARGS="-rabbit loopback_users []" rabbitmq:4-management
```

- 管理页面为 `http://localhost:15672`，默认用户名与密码均为 `guest`。
- 上述命令允许宿主机使用 `guest` 连接，并将端口绑定到宿主机的 `127.0.0.1`，用于本地开发。
- 本地部署时，`[rabbitmq].url` 填写 `amqp://guest:guest@localhost:5672/%2f`。
- 使用反馈功能前，需要在管理页面完成以下配置：
  1. 在 Exchanges 中选择虚拟主机 `/`，创建名为 `feedback` 的 Exchange，Type 选择 `fanout`，Durability 选择 `Durable`，Auto delete 选择 `No`。名称应与 `[rabbitmq].feedback_exchange` 一致。
  2. 在 Queues and Streams 中选择虚拟主机 `/`，创建名为 `feedback_queue` 的队列，Durability 选择 `Durable`，Auto delete 选择 `No`。
  3. 打开 `feedback` Exchange 的详情页，在 Bindings 中将其绑定到 `feedback_queue`，Routing key 留空。
- 消费端需要订阅 `/` 虚拟主机下的 `feedback_queue`。后端只负责发布消息，不会自动创建上述 Exchange、队列或绑定，也不会启动消费者；仅创建 Exchange 而没有绑定队列时，反馈消息不会进入消费队列。
- 连接是懒加载的，也不会自动重连——所以 RabbitMQ 重启后后端要跟着重启。

#### GreptimeDB（可选）

用于接收后端上报的 Trace 数据，以便分析请求链路和请求耗时。未配置该地址时，不会上报 Trace 数据，仅保留 stdout 日志输出。

- 本地部署方式如下：

```bash
docker run -d --name greptimedb -p 127.0.0.1:4000:4000 -v greptimedb-data:/tmp/greptimedb greptime/greptimedb:v1.1.4 standalone start --http-addr 0.0.0.0:4000
```

- 上述命令通过 Docker 数据卷保存数据，HTTP 服务端口为 `4000`。
- 配置在 `[observability]` 的 `endpoint`。本地部署填写 `http://localhost:4000/v1/otlp` 即可。
- `public.opentelemetry_traces` 表会在首次收到 Trace 数据后创建。因此，服务刚启动时查询不到该表属于正常现象。

#### Captcha 验证码服务（可选）

仅在调试“大物实验平台”相关功能时需要额外部署验证码识别（OCR）服务。

- 源码仓库为 [captcha_service](https://github.com/qnxg/captcha_service)
- 本地部署方式如下：

```bash
cd captcha_service
docker build -t captcha_service .
docker run -d -p 5000:5000 captcha_service
```

- 配置在 `[captcha]` 的 `captcha_url`，服务地址结尾不要添加斜杠，例如本地地址为：`http://localhost:5000`。

## 配置

运行项目之前，需要完成相关配置。
把 `config/config_example.toml` 复制并另存为 `config/config.toml`，按照字段提示手动填入信息。

本地开发时，通常只需配置 MySQL 和 Redis；RabbitMQ、GreptimeDB、Captcha 等组件根据实际调试需求进行配置。  

建议你将 `[server].log_level` 设为 `debug` 以方便调试（日志固定 pretty 输出到 stdout）。

将项目根目录下的 `example.env` 复制并另存为 `.env`，取消 `DATABASE_URL` 行的注释，填写 SQLx 编译期静态检查使用的数据库连接地址，通常应与 `config.toml` 中的 MySQL 配置保持一致。使用 Compose 的本地默认配置时，该行可填写为：

```dotenv
DATABASE_URL=mysql://root:root@localhost:3306/weihuda
```

在编译过程中，sqlx 会读取 `.env` 中的数据库连接信息，并连接数据库对代码中的 SQL 语句进行编译期检查。如果无法连接该数据库，相关 sqlx 编译期检查将无法完成，从而可能导致项目无法通过编译。

`config/frontend_private.pem` 为用于解密前端 RSA 加密密码的私钥。项目使用 RSA 2048 位密钥对（PKCS#8），公钥与私钥需配套使用。密钥对请按项目要求自行生成，并将私钥保存至 `config/frontend_private.pem`。


## 运行

执行 `cargo run` 即可运行。

使用示例中的监听地址时，启动后可在浏览器访问 `http://localhost:8000/ping`，响应应包含 `I am fine!`。这一步仅验证 HTTP 服务可用，数据库、Redis 和各业务接口仍需分别验证。如果修改了 `[server].address`，则请使用对应的地址和端口。

初次运行项目，Cargo 需要下载项目所需要的各种依赖库，有可能会出现等待时间过长的情况（比如 VS Code 打开项目之后一直停留在 fetching metadata 状态），出现这种情况的话你可能需要配置 Cargo 的镜像。


