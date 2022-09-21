# 上应小风筝 API v2(WIP)

## 概要

本项目旨在为上海应用技术大学的学生提供校园信息整合与管理服务，项目背景详情见 [上应小风筝](https://github.com/SIT-Yiban/kite-microapp) 项目仓库。

后端 API 为整个项目提供接口支持和数据处理。由于经费有限，尽可能需要一个资源占用小的后端服务，开发者希望它能在单核 1G 内存的机器上流畅运行，并承载和选课阶段差不多的访问量。 在之前的测试中，能稳定应对 1k
左右的并发量，并保持低内存占用。

该分支是 2.0 版本的服务端。由于学校要求导致的业务调整，上应小风筝将切换为 App 模式运营，一些功能（如账户系统）需要进行重构，
如教务模块（成绩查询、课表查询等）将下线，相关代码功能到 [kite-app](https://github.com/SIT-Yiban/kite-app) 项目并使用 Dart 语言重写。
当前的服务端仅提供基本的服务，运营支持如软件更新、使用统计，业务逻辑如公告、电费查询，并会添加查给分、二手交易等功能。

## 功能

- [ ] 电费查询
- [ ] 空教室查询
- [ ] 二手闲置交易
- [ ] 入学信息查询
- [ ] 失物招领
- [ ] 应用公告
- [ ] 应用运营统计
- [ ] 应用更新

## 环境配置

### 数据库配置

请先部署好数据库，可以参考 [配置文档](docs/数据库配置.md)。考虑到可能的兼容性问题，建议数据库版本不低于 PostgreSQL 13.2。推荐使用 DataGrip 进行后续的数据库管理操作。

### 编译

请先确保系统中已预装有 rust 编程环境（rustc、cargo等），并已连接上互联网。

下载并编译：

```shell
git clone https://github.com/SIT-Yiban/kite-server.git -b v2
cd kite-server
cargo build
```

同时修改根目录下 `kite.example.toml` 文件。默认如下：

```toml
# HTTPS API service address.
bind = "0.0.0.0:443"
# Postgresql connection string.
db = "postgresql://user:password@address:port/database"
# Token secret for API.
secret = "secret"
# Directory path should be end with "\"
attachment = "D:\\tmp\\"
# secret for weather APi
qweather_key = "secret"


```

执行下面命令即可运行，目标二进制文件存放在 `target` 目录下。

```shell
cargo run
```

## 有关项目

| 项目         | 说明             |
| ------------ | ---------------- |
| [kite-agent](https://github.com/sunnysab/kite-agent) | 后端数据抓取工具 |
| [kite-protocol](https://github.com/sunnysab/kite-protocol) | 通信协议库（已废弃）  |
| [kite-string](https://github.com/SIT-Yiban/kite-string) | 校园网爬虫工具 |

## 如何贡献

非常欢迎你的加入！[提一个 Issue](https://github.com/SIT-Yiban/kite-server/issues/new) 或者提交一个 Pull Request。

如果您有意见或建议，可以联系我们。

## 开源协议

[GPL v3](https://github.com/SIT-Yiban/kite-server/blob/master/LICENSE) © 上海应用技术大学易班 sunnysab

除此之外，您不能将本程序用于各类竞赛、毕业设计、论文等。
