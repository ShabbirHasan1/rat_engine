# 自签名证书 gRPC TLS 测试指南

本指南介绍如何使用 RAT Engine 的自签名证书 gRPC 示例进行本地测试。

## 概述

RAT Engine 提供了两个用于测试自签名证书的 gRPC 示例：

- `grpc_tls_unary_server_selfsigned.rs` - 服务端，自动生成带 SAN 的自签名证书
- `grpc_tls_unary_client_selfsigned.rs` - 客户端，跳过证书验证

## 快速开始

### 1. 启动服务端

```bash
cargo run --example grpc_tls_unary_server_selfsigned
```

服务端会自动：
- 检查证书文件是否存在
- 不存在时生成带 SAN 的自签名证书
- 监听 50051 端口

输出示例：
```
🚀 RAT Engine gRPC + TLS 服务端（自签名证书）
=============================================
绑定: 0.0.0.0:50051

📜 生成自签名证书...
✅ 证书生成成功
加载证书: examples/certs/localhost.pem
  ✓ 域名: localhost
🌐 [服务器] 不启用 mTLS（单向认证）
证书加载完成，共 1 个域名

📡 gRPC 服务:
   /hello.HelloService/Hello
   /ping.PingService/Ping
```

### 2. 运行客户端测试

新开终端：

```bash
cargo run --example grpc_tls_unary_client_selfsigned --features client
```

输出示例：
```
🔌 gRPC + TLS 客户端（自签名证书）
==================================
连接地址: https://localhost:50051

🔧 [TLS] 开始创建 TLS 配置，h2c_mode=true, h2c_over_tls=false, has_mtls=false
✅ 客户端创建成功

📤 测试 Hello 服务:
✅ Hello 请求成功:
   消息: 你好，RAT Engine 用户！欢迎使用 RAT Engine gRPC + TLS！
   时间戳: 1767598155

📤 测试 Ping 服务:
✅ Ping 请求成功:
   响应: Pong: Hello from client!
   时间戳: 1767598155

👋 客户端已关闭
```

## 证书说明

### 自动生成的证书配置

服务端会自动生成包含以下 Subject Alternative Name (SAN) 的证书：

| 类型 | 值 |
|------|-----|
| DNS | localhost |
| DNS | *.localhost |
| IP | 127.0.0.1 |

### 证书文件位置

生成后证书保存在：

- 证书：`examples/certs/localhost.pem`
- 私钥：`examples/certs/localhost-key.pem`
- 配置文件：`examples/certs/openssl.cnf`（临时）

### 手动生成证书

如需手动生成证书，可使用以下命令：

```bash
openssl req -x509 -newkey rsa:2048 \
    -keyout examples/certs/localhost-key.pem \
    -out examples/certs/localhost.pem \
    -days 365 -nodes \
    -subj "/CN=localhost" \
    -addext "subjectAltName=DNS:localhost,DNS:*.localhost,IP:127.0.0.1"
```

## 客户端配置说明

客户端使用 `h2c_mode()` 方法跳过 TLS 证书验证，适用于：

- 本地开发测试
- 自签名证书环境
- 内部网络测试

```rust
let mut client = RatGrpcClientBuilder::new()
    .connect_timeout(Duration::from_secs(5))?
    .request_timeout(Duration::from_secs(10))?
    .http2_only()
    .h2c_mode()  // 跳过证书验证
    .build()?;
```

**注意**：生产环境应使用受信任的证书颁发机构（CA）签发的证书，并配置正确的证书验证。

## 可用的 gRPC 服务

| 服务路径 | 方法 | 说明 |
|---------|------|------|
| `/hello.HelloService/Hello` | Unary | 欢迎消息服务 |
| `/ping.PingService/Ping` | Unary | ping-pong 测试服务 |

## 一元请求示例数据

### HelloRequest / HelloResponse

```rust
// 请求
HelloRequest { name: "RAT Engine 用户" }

// 响应
HelloResponse {
    message: "你好，RAT Engine 用户！欢迎使用 RAT Engine gRPC + TLS！",
    timestamp: 1767598155
}
```

### PingRequest / PingResponse

```rust
// 请求
PingRequest { message: "Hello from client!" }

// 响应
PingResponse {
    pong: "Pong: Hello from client!",
    timestamp: 1767598155
}
```

## 故障排除

### 证书验证失败

如果遇到证书验证错误：

1. 确保服务端已生成证书
2. 检查证书文件权限
3. 确认客户端使用 `h2c_mode()` 配置

### 连接被拒绝

1. 确认服务端正在运行
2. 检查端口 50051 是否被占用
3. 确认防火墙允许连接

### 证书不受信任

开发环境预期行为。客户端配置了跳过证书验证，如需信任证书：

- 将证书添加到系统信任存储
- 或在浏览器中导入证书

## 相关文档

- [gRPC 综合示例说明](../examples/grpc_comprehensive_example.rs)
- [证书管理文档]()
- [HAProxy 配置指南](haproxy_configuration.md)
