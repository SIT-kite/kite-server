## 上应小风筝 Restful api

## 项目背景

## 功能

## 安装

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
```

微信相关接口（微信登录）需要填写 `appid` 和 `secret` 后才能使用。
运行下面命令即可运行。

```shell
cargo run
```


## 有关项目

小程序代码和管理页面代码待有关同学补充。



## 如何贡献

非常欢迎你的加入！[提一个 Issue](https://github.com/sunnysab/kite-server/issues/new) 或者提交一个 Pull Request。
如果您有意见或建议，可以[联系我们](mailto:sunnysab@yeah.net)。



## 开源协议

[GPL](https://github.com/sunnysab/kite-server/blob/master/LICENSE) v3 © 上海应用技术大学易班 sunnysab

您也不能将本程序用于各类竞赛、毕业设计、论文等。