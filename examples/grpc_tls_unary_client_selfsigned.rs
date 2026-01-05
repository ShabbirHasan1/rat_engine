//! gRPC + TLS 客户端（一元请求）- 连接到使用自签名证书的服务端
//!
//! 服务端使用自签名证书时，客户端需要跳过证书验证

use rat_engine::client::grpc_client::RatGrpcClient;
use rat_engine::client::grpc_builder::RatGrpcClientBuilder;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use bincode::{Encode, Decode};

/// Hello 请求
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct HelloRequest {
    pub name: String,
}

/// Hello 响应
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct HelloResponse {
    pub message: String,
    pub timestamp: u64,
}

/// Ping 请求
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PingRequest {
    pub message: String,
}

/// Ping 响应
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PingResponse {
    pub pong: String,
    pub timestamp: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🔌 gRPC + TLS 客户端（自签名证书）");
    println!("==================================");
    println!("连接地址: https://localhost:50051");
    println!();

    // 创建 gRPC 客户端
    // 使用 h2c_mode() 跳过 TLS 证书验证（适用于自签名证书）
    let mut client = RatGrpcClientBuilder::new()
        .connect_timeout(Duration::from_secs(5))?
        .request_timeout(Duration::from_secs(10))?
        .max_idle_connections(5)?
        .http2_only()
        .user_agent("rat-engine-grpc-tls-client/1.0")?
        .disable_compression()
        .h2c_mode()  // 跳过证书验证（测试自签名证书）
        .build()?;

    println!("✅ 客户端创建成功");
    println!();

    // 测试 Hello 服务
    println!("📤 测试 Hello 服务:");
    let hello_request = HelloRequest {
        name: "RAT Engine 用户".to_string(),
    };

    match client.call_typed_with_uri::<HelloRequest, HelloResponse>(
        "https://localhost:50051",
        "hello.HelloService",
        "Hello",
        hello_request,
        None,
    ).await {
        Ok(response) => {
            println!("✅ Hello 请求成功:");
            println!("   消息: {}", response.data.message);
            println!("   时间戳: {}", response.data.timestamp);
        }
        Err(e) => {
            println!("❌ Hello 请求失败: {:?}", e);
        }
    }

    println!();

    // 测试 Ping 服务
    println!("📤 测试 Ping 服务:");
    let ping_request = PingRequest {
        message: "Hello from client!".to_string(),
    };

    match client.call_typed_with_uri::<PingRequest, PingResponse>(
        "https://localhost:50051",
        "ping.PingService",
        "Ping",
        ping_request,
        None,
    ).await {
        Ok(response) => {
            println!("✅ Ping 请求成功:");
            println!("   响应: {}", response.data.pong);
            println!("   时间戳: {}", response.data.timestamp);
        }
        Err(e) => {
            println!("❌ Ping 请求失败: {:?}", e);
        }
    }

    println!();

    // 关闭客户端
    client.shutdown().await;
    println!("👋 客户端已关闭");

    Ok(())
}
