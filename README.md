# Eussh

<img src="src-tauri/icons/128x128.png" width="96" height="96" alt="Eussh">

**Eussh** 是一个使用 Tauri v2 构建的跨平台 SSH 客户端，界面简洁、性能出色，支持多标签页管理、文件管理、批量命令执行以及主机密钥验证。

## 📸 截图

<table>
<tr><td><img src=".pic/p2_en.webp"></td><td rowspan="3" valign="top"><img src=".pic/p6.webp"></td></tr>
<tr><td><img src=".pic/p4_en.webp"></td></tr>
<tr><td><img src=".pic/p5_en.webp"></td></tr>
</table>

## ✨ 功能特性

- **Activity Bar** — VS Code 风格左侧图标栏，快速切换服务器列表、批量执行、设置面板
- **多服务器** — 同时连接多个服务器，一键切换，各服务器终端完全独立
- **多标签页** — 一个服务器可同时打开多个终端和文件管理器
- **服务器总览** — CPU、内存、磁盘、交换分区实时监控 + 世界地图服务器定位 + 全部 IP 展示
- **文件管理器** — 列表/图标视图、上传/下载、拖拽上传、右键菜单、权限管理
- **批量执行** — 多选已连接服务器，同时对多台机器执行相同命令，结果一键复制
- **主机密钥验证** — TOFU（Trust-On-First-Use），首次连接确认指纹，自动存入 known_hosts，密钥变更时警告
- **断线重连** — 网络闪断时自动指数退避重连（最多 5 次），重连期间保留终端输出
- **SSH 连接** — 密码和私钥认证（纯 Rust `russh` 实现，无 C 依赖）
- **界面自定义** — 标题栏（macOS / Windows 11 风格）、底栏、6 种终端配色、8 种强调色
- **数据安全** — 连接配置 AES-256-GCM 加密存储，主机密钥持久化校验
- **实时流量** — 底栏显示服务器上下行流量速率
- **精确延迟** — 独立通道 ping 检测，不受命令队列阻塞
- **可拖拽侧栏** — Sidebar 宽度自由调整（160–480px），持久化记忆
- **多语言** — 中文 / English
- **版本更新** — 启动时自动检查 GitHub Release，一键跳转下载

## 📦 安装

### 从 Release 下载

前往 [Releases](https://github.com/eussh/eussh/releases) 页面下载对应平台的安装包。

- **Windows**: `.msi` 和 `.exe` 安装程序
- **macOS**: `.dmg` 磁盘映像
- **Linux**: `.deb` 和 `.AppImage` 包

### 从源码构建

**环境要求：**

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/) stable (MSVC on Windows)
- 系统 WebView 组件（Windows 10+ 自带、macOS 自带、Linux 需 `libwebkit2gtk-4.1`）

```bash
git clone https://github.com/eussh/eussh.git
cd eussh
npm install
npx tauri dev      # 开发模式
npx tauri build    # 生产构建
```

## 🖥 使用指南

### 添加与连接服务器

点击左侧 Activity Bar 的服务器图标，在侧边栏底部点击 **+** 按钮，填写服务器信息。点击已保存的服务器即可连接。

连接成功后在侧边栏显示状态背景色（绿色 = 已连接、黄色 = 连接中、红色 = 断线），点击可查看**服务器总览**页面。

### 主机密钥验证

首次连接时会弹出指纹确认对话框，可选择**本次信任**或**信任并记住**。信任的密钥存入 `known_hosts`，后续连接自动校验。若服务器密钥变更，会以红色警告提示。

### 终端操作

- 点击标签栏的 **+** → **新建终端** 打开新终端
- 终端支持完整的 PTY 交互（xterm.js）
- 右键粘贴、复制
- 多服务器终端完全独立，切换无干扰

### 文件管理

- 点击标签栏的 **+** → **文件管理** 或总览页的按钮
- 双击文件夹进入，双击文件下载
- 右键菜单：下载、复制、剪切、粘贴、创建副本、重命名、删除、权限设置
- 支持从本地拖拽文件/文件夹上传
- 文件夹下载将打包为 `.tar.gz`

### 批量执行

点击 Activity Bar 的终端图标进入批量执行视图：
- 勾选目标服务器
- 输入命令（支持 `{{host}}` `{{hostname}}` 模板变量）
- 点击执行，每个服务器的结果独立展示，支持一键复制

### 设置

点击 Activity Bar 的齿轮图标，在侧边栏中配置：
- 通用：版本更新检测、调试日志
- 外观：主题、强调色、标题栏样式、底栏样式、语言
- 终端：字体、字号、光标样式、配色方案
- 监控：系统刷新间隔、延迟检测间隔、流量监控

## 🏗 技术栈

| 层 | 技术 |
|---|---|
| 桌面框架 | [Tauri v2](https://v2.tauri.app/) |
| 前端 | Vue 3 + Vite + Pinia + Tailwind CSS |
| 终端 | [xterm.js](https://xtermjs.org/) |
| SSH | [russh](https://github.com/warp-tech/russh) (纯 Rust) |
| 加密 | AES-256-GCM + PBKDF2 |

**纯 Rust 依赖链** — 项目不依赖任何 C 编译工具，所有 SSH 和加密操作均由纯 Rust 实现完成。

## 📄 开源许可

本项目采用 [MIT License](LICENSE) 开源。

---

Made with ❤️ using Tauri & Vue
