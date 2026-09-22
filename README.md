# 湖南大学微生活后端

## 概述

湖南大学微生活微信小程序的后端服务，提供课表、成绩查询、校园卡余额、电费查询、校园网流量、考试安排等校园数据的查询接口。

基于 Rust + [Salvo](https://salvo.rs) 构建，校内数据的抓取与解析由 [`hnu_query`](https://github.com/qnxg/hnu_query) 库统一完成。

## 快速开始

环境准备、依赖配置与启动方式，详见 [部署与运行](docs/deploy.md)。

## 贡献

欢迎贡献代码。请先阅读 [贡献指南](docs/contributing.md)，了解项目结构、分支规范、提交规范与 CI 要求。

## 反馈

你可以通过如下方式反馈问题：

1. [Github Issue](https://github.com/qnxg/weihuda_backend/issues)，详见 [contributing.md](docs/contributing.md) 的 `Issue 报告规范` 部分
2. [湖大微生活用户交流群](https://qm.qq.com/q/BA6FQbxXBS)，QQ 群号为 1051502207。在群内交流可获得更快的答复。
