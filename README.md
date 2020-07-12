# 上应小风筝 Restful API

## 项目概要

本项目旨在为上海应用技术大学的学生提供校园信息整合与管理服务。

## 功能

- [x] 登录模块
- [ ] 活动与签到
- [ ] 易取
- [ ] 第二课堂
- [ ] 课表查询与选课
- [ ] 二手闲置交易
- [x] 入学信息查询
- [ ] 失物招领
- [ ] 每日学习打卡

## 项目设计

TODO: 项目设计文档。链接至 `docs/`

## 目标平台

阿里云 ECS（1G 1C） Debian 10

## 安装

请先确保系统中已预装有 rust 编程环境（rustc、cargo等），并已连接上互联网。

下载并编译：

```shell
git clone https://github.com/sunnysab/kite-server.git
cd kite-server
cargo build
```

同时修改根目录下 `kite.toml.example` 文件。默认如下：

```toml
bind_addr = "0.0.0.0:80"
db_string = "postgresql://user:password@address:port/database"
jwt_string = "secret"
wechat_appid = "111"
wechat_secret = "111"
# Directory path should be end with "\"
attachment_dir = "D:\\tmp\\"
```

微信相关接口（微信登录）需要填写 `appid` 和 `secret` 后才能使用。
执行下面命令即可运行，目标二进制文件存放在 `target` 目录下。

```shell
cargo run
```

## 源码阅读


## 有关项目

| 项目         | 说明             |
| ------------ | ---------------- |
| kite-crawler | 后端数据抓取工具 |

小程序代码和管理页面代码待有关同学补充。



## 如何贡献

非常欢迎你的加入！[提一个 Issue](https://github.com/sunnysab/kite-server/issues/new) 或者提交一个 Pull Request。

如果您有意见或建议，可以联系我们。



## 开源协议

[GPL v3](https://github.com/sunnysab/kite-server/blob/master/LICENSE) © 上海应用技术大学易班 sunnysab

除此之外，您也不能将本程序用于各类竞赛、毕业设计、论文等。