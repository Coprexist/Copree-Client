<div align="center">

# AIsChat Client

**AIsChat 桌面与 Android 客户端**

[![AIsChat](https://img.shields.io/badge/AIsChat-主仓库-blue)](https://github.com/Coprexist/Copree)
[![License](https://img.shields.io/badge/license-MIT-green)](https://opensource.org/licenses/MIT)
[![Tauri](https://img.shields.io/badge/Tauri-2-purple)](https://tauri.app/)
![Windows](https://img.shields.io/badge/Windows-支持-0078D6)
![macOS](https://img.shields.io/badge/macOS-支持-999999)
![Linux](https://img.shields.io/badge/Linux-支持-FCC624)
![Android](https://img.shields.io/badge/Android-APK-3DDC84)

</div>

<br>

---

<br>

## 这是什么？

**AIsChat Client 是 [AIsChat](https://github.com/Coprexist/Copree) 的客户端**，基于 [Tauri 2](https://tauri.app/) 打造。

AIsChat 是一个开源 AI 群聊框架——AI 拥有自己的状态、记忆和节奏，群聊里能长出活的世界（群视界）。这个客户端把 AIsChat 网页应用装进原生壳里：不用开浏览器、不用记网址，一个图标点开就是你的 AI 世界。

**客户端不连接固定服务器。** 启动页是一张实例列表，地址由使用者自己填写：可以是官方实例、自己部署的实例，也可以是局域网里的 NAS。桌面端与 Android 共用同一套启动页。

<br>

---

<br>

## 特性

| 能力 | 说明 |
|------|------|
| **多实例** | 启动页填写任意实例地址，列表保存在本机，随时切换 |
| **连接前探测** | 连接前请求实例的健康检查，显示版本号；连不上时给出具体原因（超时 / 端口未开放 / 不是 AIsChat） |
| **原生窗口** | 独立应用，任务栏图标、系统级窗口管理 |
| **无浏览器干扰** | 全屏打开，不显示地址栏与标签页 |
| **多平台** | Windows / macOS / Linux / Android 一套代码 |

<br>

---

<br>

## 快速开始

### 本地开发

```bash
git clone https://github.com/Coprexist/Copree-Client.git && cd Copree-Client
npm install      # 安装依赖
npx tauri dev    # 桌面端开发模式
```

> 需要 [Rust](https://www.rust-lang.org/) 工具链与 Node.js 18+。

### Android

需要 JDK 17、Android SDK（platform 36、build-tools 36.0.0）与 NDK 26.1.10909125，以及其他环境变量：

```bash
export JAVA_HOME=/path/to/jdk-17
export ANDROID_HOME=/path/to/android-sdk
export NDK_HOME=$ANDROID_HOME/ndk/26.1.10909125

npx tauri icon icons/icon.png     # 生成平台图标（src-tauri/icons 是生成物）
npx tauri android init            # 首次生成 Android 工程（已提交，通常不需要）
npx tauri android build --apk                          # 产出 APK（默认含全部 ABI）
npx tauri android build --apk -t aarch64 armv7          # 只含真机 ABI，体积更小（CI 用这条）
```

产物位于 `src-tauri/gen/android/app/build/outputs/apk/universal/release/`，未签名；签名方式见下节。
签名后的包要能被后续版本覆盖安装，就必须始终使用同一个 keystore——本地自测与 CI 发布请共用一份。

<br>

---

<br>

## 自动构建与发布

`.github/workflows/android.yml` 在 GitHub Actions 上构建 APK：

- 手动触发（workflow_dispatch）：构建并上传 APK artifact；
- 推送 `v*` 标签：构建、签名，并附加到对应的 GitHub Release。

签名密钥通过仓库 Secrets 提供：

| Secret | 说明 |
|--------|------|
| `ANDROID_KEYSTORE_BASE64` | keystore 文件的 base64 编码 |
| `ANDROID_KEYSTORE_PASSWORD` | keystore 密码 |
| `ANDROID_KEY_ALIAS` | 密钥别名 |
| `ANDROID_KEY_PASSWORD` | 密钥密码 |

未配置密钥时，CI 会生成一次性密钥签名，产物仍可安装，但仅用于测试：换用正式密钥后无法覆盖安装，需先卸载。

生成 keystore：

```bash
keytool -genkeypair -v -keystore release.jks -alias aischat \
  -keyalg RSA -keysize 2048 -validity 10000
base64 -w0 release.jks   # 填入 ANDROID_KEYSTORE_BASE64
```

<br>

---

<br>

## 已知限制

- **返回键行为**：实例页按返回键回到启动页，启动页再按一次退到后台（与微信一致，避免 Tauri 结束进程时 WebView 线程 Abort）。
- **Android 允许明文 HTTP**：自建实例常以 `http://192.168.x.x:5227` 的形式暴露，因此 release 包放开了 Cleartext 限制。如果对外发布，请使用 HTTPS。
- **iOS 未构建**：Tauri 的 iOS 目标需要 macOS 与 Xcode，当前没有对应产物。
- **远程调试只在 debug 包开启**：`tauri` 的 `devtools` 特性不写进依赖，release 包因此不再开放 WebView 远程调试；需要调试时用 `npx tauri dev`（debug 构建会自动开启）。

<br>

---

<br>

## 关联项目

| 项目 | 说明 |
|------|------|
| [**AIsChat**](https://github.com/Coprexist/Copree) | 主项目——开源 AI 群聊框架，群聊即世界的入口 |
| [**AIsChat Community**](https://github.com/Coprexist/Copree-Community) | 世界分享社区——AIsChat 世界商城的 GitHub 数据源 |

<br>

---

<br>

## 许可证

MIT License · 自由使用、修改和分发，保留原作者署名。

<br>

---

<br>

**作者**: Coprexist 团队 · 欢迎提交 [Issue](https://github.com/Coprexist/Copree/issues) 或 Pull Request。