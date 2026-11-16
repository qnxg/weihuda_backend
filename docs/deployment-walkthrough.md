# 猴子也能学会的部署方式（部署教程口语化版）

> 本教程以 Windows + PowerShell 为例，面向初次接触后端部署的开发者。
>
> 如果你已经熟悉相关工具，可以直接阅读[部署与运行](deploy.md)。

**什么**，你是否在阅读文档后依旧晕头转向，束手无策？面对 Docker、数据库和各种配置文件完全不知道从哪里下手？  

那么，你来对地方了！

本教程面向有一定 Rust 基础、但尚不熟悉后端项目部署流程的开发者。从准备环境、启动中间件，到配置项目和测试接口，笔者会尽可能用通俗的语言，带你一步步把微生活后端跑起来。

如果你已经熟悉其中的某些步骤，也可以通过目录直接跳转。

~~不敢保证~~ 即使是猴子，也能跟着教程完成部署！

## 环境准备

### Rust
微生活的后端使用 rust 进行编写，因此需要你至少对 rust 有一定了解  
因此我们默认你已经成功配置好了 rust 的开发环境 

在正式开始之前，不妨先检查一下你的 Rust 环境是否已经准备就绪。

打开 PowerShell，依次执行：

```powershell
rustc --version
cargo --version
```

如果两个命令都能正常显示版本号，说明 Rust 编译器和 Cargo 已经可以正常使用。

如果提示找不到命令，可以先参考 [Rust 官方安装指南](https://www.rust-lang.org/tools/install) 完成安装。

### Git

Git 是一个版本控制工具，可以用来下载项目代码、记录修改历史，并与其他开发者协作

你可以前往[Git 官方下载页面](https://git-scm.com/downloads)，并按照提示下载

### Docker

Docker 是一个容器工具，可以把软件及其运行所需的依赖打包起来，在相对独立的环境中运行。这样既能减少“换台电脑就运行不了”的问题，也能降低不同软件之间发生依赖冲突的概率。

在微生活项目中，后端需要使用 MySQL、Redis、RabbitMQ 等服务。如果逐个手动安装，就需要分别处理版本、配置和启动方式。使用 Docker 后，可以直接通过镜像创建容器，快速运行这些服务，减少环境搭建的步骤。  

可以把镜像理解为软件的安装模板，把容器理解为根据模板启动的软件实例。容器中的服务通过端口映射供后端访问，数据库等需要长期保存的数据则通过数据卷保存。  

#### 1. 准备 WSL 2

WSL 可以理解为 Windows 提供的 Linux 运行环境，Docker Desktop 可以借助它运行本项目需要的 Linux 容器。

打开 **终端（管理员）**或 **PowerShell（管理员）**，执行：

```powershell
wsl --install --no-distribution
```

这会安装 WSL，且不额外安装 Ubuntu 等发行版。按照提示完成操作即可  

如果已经安装了 WSL，可以执行下面的命令更新：

```powershell
wsl --update
```

#### 2. 下载 Docker Desktop

打开 [Docker Desktop 的 Windows 安装页面](https://docs.docker.com/desktop/setup/install/windows-install/)，下载安装程序。

- 普通 Intel 或 AMD 电脑选择 **x86_64 / AMD64**。
- Windows ARM 电脑选择 **Arm**。

如果出现选择后端的选项，使用 **WSL 2**。不同版本的安装界面可能略有区别，保留 WSL 2 的默认选择即可。

打开 Docker Desktop，完成首次启动提示，等待 Docker 引擎启动完成。

> **注意：** 安装完成不代表 Docker 已经运行。后面启动中间件时，需要先打开 Docker Desktop，并等待引擎就绪。

如果提示 WSL 需要更新，执行：

```powershell
wsl --update
```

然后重新打开 Docker Desktop。

#### 3. 检查安装结果

重新打开 PowerShell，依次执行：

```powershell
docker --version
docker compose version
docker info
```

前两个命令应该显示版本号；`docker info` 应该能显示包含 **Server** 部分的信息，且没有连接失败的报错。

还可以运行一个测试容器：

```powershell
docker run --rm hello-world
```

第一次执行会先下载测试镜像。如果最后看到：

```text
Hello from Docker!
```

说明 Docker 已经可以下载镜像并运行容器，可以继续下一步了。

## 下载项目

环境准备完毕，接下来终于可以把项目拉下来了。

先找一个你打算存放代码的文件夹，在这个文件夹中打开 PowerShell，执行：

```powershell
git clone https://github.com/qnxg/weihuda_backend.git
cd weihuda_backend
```

若你使用的是自己的 Fork，把地址换成你自己的仓库地址即可。

> **小提示：** 后文若没有额外说明，所有命令都在项目根目录执行，也就是能看到 `Cargo.toml`、`src` 和 `config` 的那个文件夹。

## 认识中间件

前面一直在说中间件，那么它到底是什么？为什么运行一个后端，还要先启动这么多东西？

后端除了处理接口请求，还需要保存数据、缓存查询结果、发送消息。这些工作可以由专门的软件完成，后端在需要时调用它们。

本文把这些配套服务统称为中间件。它们作为独立的程序运行，后端通过地址和端口与它们通信。下面介绍本项目会用到的几个服务。

### MySQL：保存业务数据

用户绑定信息、问题反馈等数据，需要有地方长期保存。MySQL 就负责这件事。

MySQL 是关系型数据库，会按照表和字段组织数据：每一行是一条记录，每一列表示记录中的某项信息。后端通过 SQL 语句来查询、添加或修改这些记录。

数据库文件需要保存到磁盘上。项目通过 Docker 数据卷保存这些文件，移除容器时只要没有删除对应的数据卷，之后创建的容器仍可以挂载并使用原来的数据。

### Redis：缓存数据

有些信息短时间内不会变化，却可能被反复查询。每次都重新去校内系统请求一遍，既耗时间，也会增加对方的压力。

这时就可以先把查询结果放进 Redis，后续请求在缓存有效时直接读取，减少重复查询。缓存过期或不存在时，再重新获取数据。

本项目中的 Redis 主要承担缓存工作。缓存不代表数据永远有效，所以偶尔看到结果没有立即变化，也要考虑是不是还在缓存有效期内。

### RabbitMQ：传递消息

用户提交问题反馈时，后端会先将反馈写进 MySQL，再通过 RabbitMQ 发送消息，交给负责后续处理的程序。

RabbitMQ 是消息队列服务。后端负责发布消息，另一个程序负责接收并处理消息，这个程序称为消费者。这样，后端就不必在一次接口请求中等待所有后续处理完成。

后面会遇到的交换机（Exchange）负责根据绑定规则将消息分发到队列（Queue），消费者再从队列中读取消息。项目已经准备好了反馈功能所需的交换机、队列和绑定配置，但消费者需要另外运行。

### GreptimeDB：保存请求链路数据

如果一个接口响应很慢，我们需要知道它经过了哪些处理步骤，以及时间主要花在哪里。

GreptimeDB 在这里用来接收后端上报的请求链路数据（Trace），帮助我们查看一次请求经过了哪些步骤、各自花了多长时间，从而排查问题。

此外，我们还准备了一个绝赞的可视化链路追踪平台，让你无需直接查询数据库，也能方便地查看请求耗时和执行过程。

至于具体怎么使用，可以先记下这个名字。等后端成功启动后，我们会在[确认请求链路成功上报](#6-确认请求链路成功上报)一节实际体验它。

它属于可选组件，不配置 Trace 上报时，后端仍可以运行，也仍会在终端输出日志。项目的 Compose 将它一起启动，是为了方便需要时调试。

### 它们和 Docker 又是什么关系？

Docker 负责把这些服务运行起来，Compose 则把它们各自需要的镜像、端口和数据卷写在一份配置里，方便我们一次性启动。

现在，Docker Desktop 已经打开，接下来还需要执行下面的 Compose 命令，MySQL 等服务才会真正开始工作。

## 启动中间件

还记得前面提到的 MySQL、Redis 等服务吗？后端需要借助它们存储数据、缓存查询结果，以及发送反馈消息。我们先让这些中间件就位，再启动后端。

### 1. 一键启动

先打开 Docker Desktop，等待引擎运行，然后在项目根目录执行：

```powershell
docker compose -f docker-compose.dev.yml up -d
```

看起来有些复杂？其实这条命令可以拆成三个部分：

- `-f docker-compose.dev.yml`：指定要使用的配置文件。
- `up`：按照配置创建并启动容器。
- `-d`：让容器在后台运行，不会一直占用当前终端。

首次启动需要下载镜像，若你觉得下载速度过慢，可以选择配置国内镜像源来加速。

这份配置会启动下面几个服务：

| 服务 | 本地端口 | 用途 |
| --- | --- | --- |
| MySQL | `3306` | 存储用户信息、反馈等数据 |
| Redis | `6379` | 缓存查询结果 |
| RabbitMQ | `5672`，管理页面为 `15672` | 发送问题反馈消息 |
| GreptimeDB | `4000` | 接收请求链路数据，方便排查问题 |

### 2. 确认启动成功

执行：

```powershell
docker compose -f docker-compose.dev.yml ps
```

如果几个服务的状态都是 `Up` 或 `running`，说明容器已经运行。接下来等待 MySQL 初始化数据库完成即可。

初始化脚本会在首次创建空数据卷时建立 `weihuda` 数据库中的表，因此无需再逐个手动建表。

> **注意：** 已有数据卷时，重启容器不会重新执行初始化脚本。若你之前创建过一个没有表的数据库，需要先确认其中是否有需要保留的数据，再处理初始化问题。不要为了“再试一次”就随手删掉数据卷。

若想确认一下初始化完成，可以进入 MySQL：

```powershell
docker compose -f docker-compose.dev.yml exec mysql mysql -uroot -p
```

提示输入密码时，输入 `root`。输入密码时屏幕上没有字符显示，这是正常现象，输完按回车即可。

进入后依次执行：

```sql
USE weihuda;
SHOW TABLES;
```

如果看到 `mini_bind`、`feedbacks` 等表名，说明表结构已经准备好了。输入 `exit;` 退出 MySQL，回到原来的终端。

### 3. RabbitMQ 还要手动配置吗？

使用项目提供的 Compose 配置时，反馈使用的 `feedback` 交换机、`feedback_queue` 队列以及它们之间的绑定会一并配置，无需再手动创建。

你可以在浏览器打开 `http://localhost:15672`，使用 `guest` / `guest` 登录管理页面，看看它们是否已经出现。

不过后端只负责发送消息，真正读取队列并处理反馈的消费者需要另外运行。

## 填写配置

中间件已经就位，但后端还不知道去哪里找它们。接下来，我们要给它写一份“通讯录”。

### 1. 复制配置文件

在项目根目录执行：

```powershell
Copy-Item config/config_example.toml config/config.toml
Copy-Item example.env .env
```

这两个文件各有用途：`config.toml` 是后端运行时读取的配置，`.env` 则为 SQLx 编译时检查数据库提供连接地址。后面会分别填写。

### 2. 填写配置文件

打开 `config/config.toml`，将对应段落修改为下面的内容。这里是修改原有段落，不要在文件末尾再粘贴一套同名段落，否则 TOML 会报重复定义。

```toml
[server]
name = "weihuda_local"
address = "127.0.0.1:8000"
log_level = "debug"

[database]
max_connections = 10
database_url = "mysql://root:root@127.0.0.1:3306/weihuda"

[redis]
redis_url = "redis://127.0.0.1:6379/"

[rabbitmq]
url = "amqp://guest:guest@127.0.0.1:5672/%2f"
feedback_exchange = "feedback"
```

MySQL 的用户名和密码都是 `root`，Redis 没有设置密码，RabbitMQ 的用户名和密码都是 `guest`。这些值对应项目 Compose 中的本地开发配置。

RabbitMQ 地址最后的 `%2f` 表示名为 `/` 的虚拟主机，先照着填写即可。

### 3. 配置密钥与其他功能

- `[captcha]`：大物实验平台使用的验证码识别服务。可以先将 `captcha_url` 写为 `http://127.0.0.1:5000`；需要调试对应功能时，再参照[部署与运行](deploy.md#captcha-验证码服务可选)启动服务，仅填写地址不会自动启动它。
- `[pow]`：保留示例配置即可，这是绑定接口使用的工作量证明校验。
- `[observability]`：用于配置请求链路追踪功能。

  如果使用刚才启动的 GreptimeDB，将 `endpoint` 设置为：

  `http://127.0.0.1:4000/v1/otlp`

  配置成功后，可以通过下面的地址访问 GreptimeDB Dashboard：

  http://127.0.0.1:4000/dashboard

  如果暂时不需要 Trace 功能，可以删除整个 `[observability]` 配置段。后端仍然可以正常运行，并继续在终端输出日志。

除了可选的 `[observability]`之外，上面提到的其他配置段，即使暂时不用对应功能，也不要直接删掉，保持和example里的默认值一致即可。

### 4. 配置编译时使用的数据库

打开根目录的 `.env`，找到 `DATABASE_URL` 那一行，去掉前面的 `#`，修改为：

```dotenv
DATABASE_URL=mysql://root:root@127.0.0.1:3306/weihuda
```

你可能会想：刚才不是已经填过数据库地址了吗，为什么还要再填一次？

因为项目使用了 SQLx 的编译期检查，它会在编译时连接数据库，检查 SQL 中的表和字段是否正确。而后端真正运行之后，使用的是 `config.toml` 中的连接地址。

所以，编译前就要把 MySQL 启动好，并且把表建好。只有一个空的 `weihuda` 数据库，依然过不了检查。

### 5. 准备前端密码解密所需的密钥

前端发送的加密密码，需要后端使用对应的 RSA 私钥来解密。若你要调试登录、绑定等涉及密码解密的流程，还需要准备这个文件：

```text
config/frontend_private.pem
```

已有配套的开发密钥时，直接将私钥放到这个位置。若要为自己的本地前后端生成一套新密钥，可以使用 OpenSSL：

```powershell
openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -out config/frontend_private.pem
openssl pkey -in config/frontend_private.pem -pubout -out config/frontend_public.pem
```

第一行生成 RSA 2048 位、PKCS#8 格式的私钥，第二行导出对应的公钥。将公钥配置到你本地前端所使用的位置，私钥保留在后端。

如果你已经按照前面的教程安装了 Git for Windows，那么它已经附带了 OpenSSL ；因此若 PowerShell 找不到 `openssl`，在项目根目录打开 Git Bash 执行上面的命令即可。

> **注意：** 公钥和私钥必须配套。仅给后端换一把新私钥，却继续使用前端原来的公钥，解密自然不会成功。私钥和真实配置都不要提交到公开仓库。

仅访问 `/ping` 不会触发这把私钥的读取，可以先验证服务启动，再回来配置前端联调。

## 启动后端

到这里，终于轮到主角登场了。

确认 Docker Desktop 还在运行，中间件也已经启动，然后在项目根目录执行：

```powershell
cargo run
```

第一次运行，Cargo 会下载依赖并编译项目，等待时间可能比较长。

如果出现的是依赖下载失败，请先根据报错检查网络；如果出现的是数据库连接或表不存在，则回头检查中间件是否已经运行、 MySQL初始化是否成功和 `.env`的配置是否有错漏。

当日志显示服务监听在 `127.0.0.1:8000`，且程序没有随即退出时，就可以验证接口了。

### 1. 待我先试探一手

在浏览器中打开：

```text
http://127.0.0.1:8000/ping
```

正常情况下，你会看到类似下面的响应：

```json
{
  "code": 200,
  "data": "I am fine!",
  "msg": "请求成功"
}
```

看到 `I am fine!`，说明 HTTP 服务已经能接收请求并返回结果了。

不过 `/ping` 不会查询 MySQL，也不会读写 Redis，所以这一步通过后，仍需要继续验证你要使用的业务接口。

### 2. 试试具体接口

笔者在此推荐你使用 ApiFox 进行接口测试

项目的接口定义可以在 [OpenAPI 文档](openapi.yaml) 中查看。先找到你要测试的接口，再确认请求方式、参数和是否需要登录。

对于需要登录的接口，要在请求头中携带 `Authorization`，值填写后端签发的 JWT 字符串。当前代码会直接解析这个值，因此不要自行在前面加上 `Bearer `。

若想了解接口具体应该如何编写，可以阅读[贡献指南中的接口编写示例](contributing.md#接口编写示例)

### 3. 通过公告接口验证 MySQL 查询

接下来，我们让后端实际调用中间件。

先保持 `cargo run` 正在运行，再打开另一个 PowerShell 终端。下面的地址都以监听 `127.0.0.1:8000` 为例；若你改过监听端口，也要一起修改。

执行：

```powershell
Invoke-RestMethod -Uri 'http://127.0.0.1:8000/announcement' -TimeoutSec 15 | ConvertTo-Json -Depth 10
```

这个接口不需要登录，会通过后端查询 MySQL 中的公告表。正常响应中的 `code` 应为 `200`，`data` 是公告数组。如果刚初始化的数据库没有公告，返回 `[]` 也是正常的。

它验证了后端运行时的数据库地址、账号密码，以及公告查询所需的表和字段。

如果请求失败，查看后端终端中的数据库连接或 SQL 错误；当前实现连接 MySQL 失败时可能直接退出后端，修正配置后需要重新执行 `cargo run`。

### 4. 通过扫码登录码接口验证 Redis 读写

我们生成一个登录码，再查询它的状态。

在 PowerShell 终端依次执行：

```powershell
$qrResult = Invoke-RestMethod -Uri 'http://127.0.0.1:8000/auth-qrcode' -TimeoutSec 15
$qrResult | ConvertTo-Json -Depth 10
$loginCode = $qrResult.data
Invoke-RestMethod -Uri "http://127.0.0.1:8000/auth-qrcode/status/$loginCode" -TimeoutSec 15 | ConvertTo-Json -Depth 10
```

第一条请求应返回 `code: 200` 和一个随机登录码；如果这一步失败，先解决报错，再执行后面的查询。

第二次请求应返回：

```json
{
  "code": 200,
  "data": "unused",
  "msg": "请求成功"
}
```

生成登录码时，后端会将它写入 Redis；查询状态时，再从 Redis 读取。这里直接调用缓存读写函数，若 Redis 失败会使接口报错。

如果查询时已经提示“找不到二维码”，重新生成一个再尝试。

### 5. 通过反馈接口验证 MySQL 写入与 RabbitMQ 投递

这一步会在本地数据库中新增一条测试反馈，并产生一条真实队列消息。

在 PowerShell 中执行：

```powershell
$testMarker = 'walkthrough-smoke-' + [guid]::NewGuid().ToString('N')
$feedbackBody = @{
    stuId = 'local-smoke-test'
    desc = $testMarker
    contact = 'local test only'
} | ConvertTo-Json
Invoke-RestMethod -Method Post -Uri 'http://127.0.0.1:8000/feedback/no_auth' -ContentType 'application/json' -Body $feedbackBody -TimeoutSec 30 | ConvertTo-Json -Depth 10
$testMarker
```

#### 基础验证

执行上面的命令后，如果接口返回 `code: 200`，说明反馈已经成功写入 MySQL。

但要注意：当前后端在 RabbitMQ 投递失败时，仍可能返回反馈成功，因此这还不能证明消息已经进入队列。

接下来打开 [RabbitMQ 管理页面](http://127.0.0.1:15672)，进入 `feedback_queue`。

使用 **Get messages** 功能查看队列中的消息，选择 **Ack message requeue true**，避免查看后直接移除消息。

如果能找到 `desc` 中包含本次 `$testMarker` 的消息，就说明本次本地测试的消息已成功进入队列。

注意：如果已有消费者正在运行，消息也可能被消费者提前取走。这时不能仅凭队列为空断定消息投递失败。

#### 进阶验证：检查数据并清理测试记录

若要进一步核对数据库写入，可以在本地 MySQL 中根据消息里的 `id` 查询：

```sql
SELECT id, `desc`, contact FROM feedbacks WHERE id = 这里替换为消息里的数字id;
```

这条 SQL 里的中文占位需要换成真实数字再执行。确认记录含有本次测试标记后，也可以按相同 `id` 删除这条测试反馈。不过，删除数据库记录不会自动删除 RabbitMQ 中的消息，两边需要分别清理。

测试结束后，在 RabbitMQ 管理页面进入 **Queues and Streams → feedback_queue**，根据队列中的内容选择清理方式：

- 如果本地队列里全是可以删除的测试消息，展开 **Purge**，点击 **Purge Messages**，即可清空等待消费的消息，队列和绑定关系会保留。
- 如果只想取出并移除少量消息，可以在 **Get messages** 中将数量设为 `1`，确认模式选择 **Ack message requeue false**。注意，这会直接移除本次取出的消息，不能先按 `$testMarker` 筛选再删除。只有确认接下来取出的消息都可以删除时才使用，不要误删其他反馈。

如果 RabbitMQ 重启过，后端也要重启，再重新提交一次测试反馈；当前连接不会自动重连。每次执行上面的提交命令都会新增一条反馈，所以不要反复点击而却忘了清理。

### 6. 确认请求链路成功上报

若你启用了 `[observability]`，可以顺手把 GreptimeDB 的最后一步测试也完成。

确认 `endpoint` 为 `http://127.0.0.1:4000/v1/otlp`，并已在修改配置后重启后端。再请求几次 `http://127.0.0.1:8000/ping`。

后端会批量上报 Trace，因此稍等几秒，再在浏览器中打开 `http://127.0.0.1:4000/dashboard`

点击网页左侧的`Traces Query`，你可以在这里看到历史请求数据。

看到本次请求的记录，则说明该链路已经跑通。若你没有启用可观测性，这一节可以跳过。

## 日常开发与收工

恭喜你🎉，如果你完成了上面的所有测试，那么你已经成功将本项目部署到本地了！（鼓掌）

修改 Rust 代码后，需要在运行后端的终端中按 `Ctrl + C` 停止程序，再执行 `cargo run`，新的代码才会生效。只点保存，正在运行的后端不会自动更新。

修改 `config.toml` 后也需要重启后端，让它重新读取配置。

需要退出时，直接按 `Ctrl + C` 停止后端，再执行：

```powershell
docker compose -f docker-compose.dev.yml stop
```

下次继续开发，先打开 Docker Desktop，再执行启动中间件的命令，最后运行 `cargo run` 即可。

如果希望移除这一组容器和网络，可以使用：

```powershell
docker compose -f docker-compose.dev.yml down
```

普通的 `down` 会保留 Compose 中声明的 MySQL 等命名数据卷。所以不要添加 `-v`，它会连数据卷一起删除。Redis 在当前 Compose 中没有配置持久化数据卷，移除并重建容器后缓存会丢失。

## 遇到问题先别急

遇到报错很正常，但是你先别急。

| 现象 | 检查方向 |
| --- | --- |
| Docker 提示无法连接引擎 | Docker Desktop 是否已打开，是否已完成启动 |
| 启动容器时报端口已被占用 | 本机是否已经运行了另一套 MySQL、Redis 等服务；若修改宿主机映射端口，记得同步修改后端连接地址和 `.env` |
| Cargo 报数据库连接失败 | MySQL 是否初始化完成，`.env` 中的地址、用户名和密码是否正确 |
| Cargo 报表或字段不存在 | 是否执行过对应版本的数据库初始化脚本，是否连接到了正确的 `weihuda` 数据库 |
| 启动时报 `No valid configuration file found!` | 是否创建了 `config/config.toml`；往上翻日志，看是文件找不到、TOML 格式错误，还是缺少字段 |
| `/ping` 打不开 | `cargo run` 是否还在运行，监听地址和访问端口是否一致 |
| 密码解密时报错 | 私钥文件路径、PKCS#8 格式，以及前后端的公私钥是否配套 |
| 反馈没有进入队列 | RabbitMQ 是否运行，`feedback` 交换机及队列绑定是否存在；若 RabbitMQ 重启过，也重启一下后端 |

如果还是解决不了，提问时尽量带上你执行的命令、完整报错和运行环境。截图或复制日志之前，记得遮住密码、令牌和私钥。

到这里，你的后端应该已经正常运行了。接下来，就可以放心地开始 ~~打工~~ 开发了
