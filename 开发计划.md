## 开发计划

账户

每个用户只能通过微信登录（管理员除外），即，普通用户必须绑定一个微信。可选绑定学号姓名。绑定学号时，须提供学号、密码（todo：少数民族姓名处理）。未绑定学号的账户，限制个人相关功能，但可以浏览公共部分，有利于演示和传播。



### 用户管理

#### 数据表设计

##### 用户信息表

用户信息表（person）用于存储用户实体，是其他用户相关表的基础。定义如下：

```sql
create table persons
(
    id           serial not null
        constraint person_pkey
            primary key,
    uid          serial not null
        constraint persons_uid_key
            unique,
    sex          int not null default 0,
    real_name    varchar(20),
    nick_name    varchar(20),
    avatar_url   varchar(100),
    avatar       varchar(64),
    profile      varchar(50),
    status       int not null,
    country      varchar(30),
    province     varchar(30),
    city         varchar(30),
    role         int    not null
);
```

##### 认证信息表

认证信息表用于存储登录方式。由于在设计中，管理员通常使用用户名/密码登录，其他人使用微信登录。为增加灵活性，参考知乎上认证相关信息的实践，单独设计本表。定义如下：

```sql
create table verifications
(
    id         serial      not null
        constraint verifications_pkey
            primary key,
    uid        serial      not null
        constraint verifications_persons_uid_fk
            references persons (uid)
            on update restrict,
    login_type int    not null,
    account    varchar(40) not null,
    credential varchar(40),
);
```

##### OA实名信息

用于关联学校OA信息的账户系统。定义如下：

```sql
create table oa_bindings
(
    id           serial  not null
        constraint oa_bindings_pkey
            primary key,
    uid          integer not null
        constraint oa_bindings_persons_uid_fk
            references persons (uid)
            on update restrict,
    student_id   char(10),
    oa_password  char(10),
    oa_certified boolean not null default false,
    class        char(8)
);
```

##### 登录记录

// TODO.

要求记录用户设备信息，登录的IP地址，登录时间。该操作使用独立接口。



#### 微信登录

##### 流程

根据微信开发文档，通过调用 `wx.login`调起授权框。用户授权登录后，小程序得到一个临时的 `code`并传给后端。后端调用`code2Session`接口，主要获取两个数据：`openid`和`session_key`，前者用来标识用户，后者用来标记会话。`session_key`的有效时长是动态的。

<img src="https://res.wx.qq.com/wxdoc/dist/assets/img/api-login.2fcc9f35.jpg" alt="img" style="zoom: 80%;" />

此后，`openid`和`session_key`由后端保管，根据[微信要求](https://developers.weixin.qq.com/miniprogram/dev/framework/open-ability/login.html#%E8%AF%B4%E6%98%8E%EF%BC%9A)，`session_key`不能返回到前端，意味着很多数据交互必须由后端服务完成。登陆过程中若不存在该用户，则自动创建用户，并返回自定义`token`。

微信API提供的用户信息主要有：

> 所谓**非敏感信息**
>
> string nickName 用户昵称
>
> string avatarUrl 用户头像
>
> number gender 用户性别（0 未知，1 男性，2女性）
>
> string country 所在国家
>
> string province 所在省份
>
> string city 所在城市
>
> string language 上述地理位置信息所用语言（en，zh_CN，zh_TW）
>
> 所谓**敏感信息**
>
> string openId 用户在此开放平台（微信、企业微信、公众号等入口）上的 UUID
>
> string unionId 全局 UUID
>
> 敏感信息需要登录态获取，非敏感信息可以在登录态失效的情况下获取，但要求用户之前必须授权过。

##### 参考

小程序登录流程：https://developers.weixin.qq.com/miniprogram/dev/framework/open-ability/login.html

数据解密规则 https://developers.weixin.qq.com/miniprogram/dev/framework/open-ability/signature.html，里面附有解密相关的 Demo.

### 开发计划

- [ ] 添加用户（40min）

- [ ] 修改用户信息（30min）

- [ ] 删除用户（20min）

- [ ] 用户权限控制（60min）

- [ ] 敏感信息过滤（文字和图片）（3h）

- [x] >  **增加用户基础信息表**（2h）
  >
  > - [ ] id
  >
  > - [ ] 真实姓名
  >
  > - [ ] 性别
  > - [ ] 个人简介
  > - [ ] 头像（初期本地存储，后期OSS）
  > - [ ] 班级
  > - [ ] 学号
  > - [ ] OA密码
  > - [ ] status

- [ ] > 增加签到活动的activities表（最小权限原则（1h）
  >
  > - [ ] activity id
  > - [ ] title
  > - [ ] description
  > - [ ] creator
  > - [ ] tags[ ]
  > - [ ] create time
  > - [ ] status

- [ ] 优化程序性能
- [ ] 做一次 benchmark
- [ ] 头像url

### 请求

code

msg

task_no