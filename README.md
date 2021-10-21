# 上应小风筝 API

## 概要

本项目旨在为上海应用技术大学的学生提供校园信息整合与管理服务，项目背景详情见 [上应小风筝](https://github.com/SIT-Yiban/kite-microapp) 项目仓库。

后端 API 为整个项目提供接口支持和数据处理。由于经费有限，尽可能需要一个资源占用小的后端服务，开发者希望它能在单核 1G 内存的机器上流畅运行，并承载和选课阶段差不多的访问量。 在之前的测试中，能稳定应对 1k
左右的并发量，并保持低内存占用。

## 功能

- [x] 课表查询
- [x] 空教室查询
- [x] 入学信息查询
- [x] 电费查询
- [x] 通讯录查询
- [x] 成绩查询
- [x] 图书馆图书检索
- [x] 图书馆图书信息查询
- [ ] 活动与签到（开发中）
- [ ] 二手书交易（开发中）
- [ ] 消费查询（开发中）
- [ ] 失物招领
- [ ] 校园地图
- [ ] 校内公告

## 环境配置

### 数据库配置

在 `sql` 目录下有一个 `initial.sql` 文件，用于初始化数据库。

请先部署好数据库，可以参考 [配置文档](docs/数据库配置.md)。考虑到可能的兼容性问题，建议数据库版本不低于 PostgreSQL 13.2。

**方式一** 在 `psql` 中导入数据库

切换到数据库脚本所在路径后，通过 `psql` 连接到对应的数据库：

```shell
cd kite-server/sql
psql -U postgres
```

进入数据库后，输入：

```shell
\i initial.sql
```

**方式二** 直接通过 `psql` 命令执行 SQL 文件

```shell
psql -U postgres -f kite-server/sql/initial.sql
```

导入完成即可。推荐使用 DataGrip 进行后续的数据库管理操作。

### 编译

请先确保系统中已预装有 rust 编程环境（rustc、cargo等），并已连接上互联网。

下载并编译：

```shell
git clone https://github.com/SIT-Yiban/kite-server.git
cd kite-server
cargo build
```

同时修改根目录下 `kite.example.toml` 文件。默认如下：

```toml
# Server config
[server]
# HTTPS API service address.
bind = "0.0.0.0:443"
# Postgresql connection string.
db = "postgresql://user:password@address:port/database"
# Token secret for API.
secret = "secret"
# Directory path should be end with "\"
attachment = "D:\\tmp\\"

# Wechat platform config. Access https://mp.weixin.qq.com for details
[wechat]
# Miniprogram appid
appid = "111"
# Secret
secret = "111"

[host]
# Bind address, for accepting connections from agents
bind = "0.0.0.0:1040"
# Max agent connections
max = 32
```

微信相关接口（微信登录）需要填写 `appid` 和 `secret` 后才能使用。 执行下面命令即可运行，目标二进制文件存放在 `target` 目录下。

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

非常欢迎你的加入！[提一个 Issue](https://github.com/sunnysab/kite-server/issues/new) 或者提交一个 Pull Request。

如果您有意见或建议，可以联系我们。

## 开源协议

[GPL v3](https://github.com/sunnysab/kite-server/blob/master/LICENSE) © 上海应用技术大学易班 sunnysab

除此之外，您不能将本程序用于各类竞赛、毕业设计、论文等。