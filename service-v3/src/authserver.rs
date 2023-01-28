/*
 * 上应小风筝  便利校园，一步到位
 * Copyright (C) 2021-2023 上海应用技术大学 上应小风筝团队
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

pub use client::Session;
pub use portal::{Credential, Portal, PortalConnector};
pub use tls::get as tls_get;

mod client;
mod portal;
mod tls;

pub mod constants {
    pub const SERVER_NAME: &str = "authserver.sit.edu.cn";
    /// 登录页. 第一次请求使用 GET 方法, 发送表单使用 POST 方法.
    pub const LOGIN_URI: &str = "/authserver/login";
    /// 访问登录后的信息页
    pub const AUTH_SERVER_HOME_URI: &str = "/authserver/index.do";
    /// 检查该用户登录是否需要验证码
    pub const NEED_CAPTCHA_URI: &str = "/authserver/needCaptcha.html";
    /// 登录时使用的 User-Agent
    pub const DESKTOP_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.99 Safari/537.36 Edg/97.0.1072.69";
    /// 验证码
    pub const CAPTCHA_URI: &str = "/authserver/captcha.html";
    /// 验证码识别服务
    pub const CAPTCHA_REORGANIZATION_URL: &str = "https://kite.sunnysab.cn/api/ocr/captcha";
}
