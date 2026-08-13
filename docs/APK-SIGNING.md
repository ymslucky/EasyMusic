# Android APK 签名配置指南

## 问题描述

APK 安装时报错 `PackageInfo is null`，根本原因：**APK 未签名**。

CI 日志显示生成的文件是 `app-universal-release-unsigned.apk`。
Android 7.0+ 拒绝安装未签名 APK，包安装器返回 `PackageInfo is null`。

## 当前修复（已完成）

`build-apk.yml` 已修改：当未配置签名密钥时，自动回退到 **debug 签名** 构建（`--debug`），
而不是生成不可安装的 unsigned release APK。Debug 签名的 APK 可以正常安装。

## 正式签名配置（推荐用于 Release）

要生成正式签名的 release APK，需要配置 4 个 GitHub Secrets：

### 1. 生成签名密钥

```bash
keytool -genkey -v \
  -keystore release.keystore \
  -alias easymusic \
  -keyalg RSA \
  -keysize 2048 \
  -validity 10000 \
  -storepass YOUR_STORE_PASSWORD \
  -keypass YOUR_KEY_PASSWORD
```

### 2. Base64 编码密钥文件

```bash
base64 -w0 release.keystore > keystore.b64
```

### 3. 配置 GitHub Repository Secrets

在 GitHub 仓库 → Settings → Secrets and variables → Actions → New repository secret：

| Secret 名称            | 值                              |
|------------------------|---------------------------------|
| `SIGNING_KEY`          | `keystore.b64` 文件的完整内容    |
| `KEYSTORE_PASSWORD`    | keystore 文件密码               |
| `KEY_ALIAS`            | 密钥别名（如 `easymusic`）      |
| `KEY_PASSWORD`         | 密钥密码                        |

### 4. 验证

配置 Secrets 后，推送代码或打 tag 触发构建。
CI 将自动：
1. 解码密钥
2. 配置 Gradle release 签名
3. 构建签名 release APK
4. 使用 `apksigner verify` 验证签名有效性

## 临时方案

如果不配置签名密钥，CI 会自动构建 **debug-signed APK**（文件名含 `-debug` 后缀）。
该 APK 可正常安装，但：
- 仅适用于测试/开发
- 使用 Android 默认 debug 证书签名
- 不能上传到 Google Play Store
