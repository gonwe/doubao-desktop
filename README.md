# doubao-desktop

一个面向 Linux 的非官方豆包桌面封装。

本项目使用 [Tauri](https://tauri.app/) 将豆包官方 Web 页面封装为桌面应用，主要用于个人 Linux 桌面环境，并方便制作和维护 AUR 软件包。

## 说明

本项目：

- 不提供豆包 AI 服务；
- 不修改或重新实现豆包核心功能；
- 不包含豆包服务端代码；
- 直接加载豆包官方 Web 页面；
- 仅提供桌面窗口、Linux 适配及相关打包配置。

主要目的是解决 Linux 下没有对应桌面客户端时的使用需求。

## 功能

- Tauri 原生桌面窗口
- 直接访问豆包官方 Web 页面
- Linux / WebKitGTK 中文字体显示优化
- WebKitGTK 图片粘贴修复
- 单实例运行：重复点击图标只聚焦已有窗口
- 站内导航留在应用窗口，站外链接交给系统浏览器
- 适合 Arch Linux / AUR 打包使用
- 保持较小的本地程序体积

## 配置

全部配置通过环境变量覆盖，均为可选：

| 变量 | 默认值 | 说明 |
| --- | --- | --- |
| `DOUBAO_URL` | `https://www.doubao.com/chat/` | 启动时加载的地址 |
| `DOUBAO_USER_AGENT` | 未设置（沿用 WebKit 原生 UA） | 覆盖 User-Agent |
| `DOUBAO_HIDE_BUTTONS` | `下载豆包电脑版` | 需要隐藏的按钮文本，逗号分隔；设为空字符串则不隐藏 |
| `DOUBAO_DEBUG` | 未设置 | 打印实际生效的配置到 stderr |

关于 `DOUBAO_USER_AGENT`：默认**不做**伪装，因为改写 UA 会改变站点看到的环境指纹，可能触发异地登录风控。
需要对比测试时可以显式指定，例如：

```sh
DOUBAO_DEBUG=1 \
DOUBAO_USER_AGENT='Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36' \
  doubao-desktop
```

窗口脚本（`doubao-desktop`）设置的 `WEBKIT_DISABLE_COMPOSITING_MODE`、`FREETYPE_PROPERTIES`、
`WEBKIT_FORCE_FONT_CONFIG` 同样只在未设置时才导出，不会覆盖用户已有的值。

## 从源码构建

```sh
# 依赖：webkit2gtk-4.1 gtk3 libsoup3 rust
cargo build --release --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

产物在 `src-tauri/target/release/doubao-desktop`。发布用的 tarball 由 CI 在打 tag 时自动组装，
布局与 AUR 包期望的一致。

## Arch Linux

本项目主要用于方便 Arch Linux / AUR 用户安装和使用。

具体安装方式以对应 AUR 软件包说明为准。

## Disclaimer

**本项目为非官方第三方项目，与豆包、字节跳动及其关联公司不存在官方合作、授权、认可或隶属关系。**

"豆包"、相关名称、商标、Logo、网站内容、界面、AI 服务及其他相关资源的知识产权归其各自权利人所有。

本仓库仅包含第三方桌面封装、Linux 适配及打包相关代码。

本项目不会对豆包官方网页、服务或内容进行重新分发，使用过程中实际加载的内容来自豆包官方服务。

如果相关权利人认为本项目存在不适当使用，请通过 GitHub Issue 联系，我会及时处理。

## License

本仓库中由项目维护者原创的桌面封装及相关代码，可按照本仓库所声明的开源许可证使用。

豆包相关商标、Logo、网页内容及服务不属于本项目开源许可证的授权范围。
