//! gRPC + TLS 服务端（一元请求）- 使用自签名证书
//!
//! 生成自签名证书命令：
//! openssl req -x509 -newkey rsa:2048 -keyout localhost-key.pem -out localhost.pem -days 365 -nodes -subj "/CN=localhost"

use rat_engine::{RatEngine, Router};
use rat_engine::server::grpc_handler::UnaryHandler;
use rat_engine::server::grpc_types::{GrpcRequest, GrpcResponse, GrpcContext, GrpcError};
use rat_engine::server::cert_manager::{CertificateManager, CertConfig, CertManagerConfig};
use serde::{Serialize, Deserialize};
use bincode::{Encode, Decode};
use std::pin::Pin;
use std::future::Future;

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

/// Hello 处理器
struct HelloHandler;

impl UnaryHandler for HelloHandler {
    fn handle(
        &self,
        request: GrpcRequest<Vec<u8>>,
        _context: GrpcContext,
    ) -> Pin<Box<dyn Future<Output = Result<GrpcResponse<Vec<u8>>, GrpcError>> + Send>> {
        Box::pin(async move {
            let hello_req: HelloRequest = match bincode::decode_from_slice(&request.data, bincode::config::standard()) {
                Ok((req, _)) => req,
                Err(e) => {
                    return Err(GrpcError::InvalidArgument(format!("解码失败: {}", e)));
                }
            };

            let response = HelloResponse {
                message: format!("你好，{}！欢迎使用 RAT Engine gRPC + TLS！", hello_req.name),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };

            let response_bytes = match bincode::encode_to_vec(&response, bincode::config::standard()) {
                Ok(bytes) => bytes,
                Err(e) => {
                    return Err(GrpcError::Internal(format!("编码失败: {}", e)));
                }
            };

            Ok(GrpcResponse {
                data: response_bytes,
                status: 0,
                message: "OK".to_string(),
                metadata: Default::default(),
            })
        })
    }
}

/// Ping 处理器
struct PingHandler;

impl UnaryHandler for PingHandler {
    fn handle(
        &self,
        request: GrpcRequest<Vec<u8>>,
        _context: GrpcContext,
    ) -> Pin<Box<dyn Future<Output = Result<GrpcResponse<Vec<u8>>, GrpcError>> + Send>> {
        Box::pin(async move {
            let ping_req: PingRequest = match bincode::decode_from_slice(&request.data, bincode::config::standard()) {
                Ok((req, _)) => req,
                Err(e) => {
                    return Err(GrpcError::InvalidArgument(format!("解码失败: {}", e)));
                }
            };

            let response = PingResponse {
                pong: format!("Pong: {}", ping_req.message),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };

            let response_bytes = match bincode::encode_to_vec(&response, bincode::config::standard()) {
                Ok(bytes) => bytes,
                Err(e) => {
                    return Err(GrpcError::Internal(format!("编码失败: {}", e)));
                }
            };

            Ok(GrpcResponse {
                data: response_bytes,
                status: 0,
                message: "OK".to_string(),
                metadata: Default::default(),
            })
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 RAT Engine gRPC + TLS 服务端（自签名证书）");
    println!("=============================================");
    println!("绑定: 0.0.0.0:50051");
    println!();

    // 证书文件路径
    let cert_path = "examples/certs/localhost.pem";
    let key_path = "examples/certs/localhost-key.pem";

    // 检查证书文件是否存在，不存在则生成
    if !std::path::Path::new(cert_path).exists() {
        println!("📜 生成自签名证书...");
        // 使用 openssl.cnf 配置文件添加 SAN
        let conf_content = r#"
[ req ]
distinguished_name = req_distinguished_name
x509_extensions = v3_req
prompt = no
default_md = sha256

[ req_distinguished_name ]
CN = localhost

[ v3_req ]
basicConstraints = CA:FALSE
keyUsage = nonRepudiation, digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth
subjectAltName = @alt_names

[ alt_names ]
DNS.1 = localhost
DNS.2 = *.localhost
IP.1 = 127.0.0.1
"#;
        std::fs::write("examples/certs/openssl.cnf", conf_content)?;

        let output = std::process::Command::new("openssl")
            .args(&[
                "req", "-x509",
                "-newkey", "rsa:2048",
                "-keyout", key_path,
                "-out", cert_path,
                "-days", "365",
                "-nodes",
                "-config", "examples/certs/openssl.cnf"
            ])
            .output();
        match output {
            Ok(o) => {
                if !o.status.success() {
                    println!("❌ 证书生成失败: {}", String::from_utf8_lossy(&o.stderr));
                    return Err("证书生成失败".into());
                }
                println!("✅ 证书生成成功");
            }
            Err(e) => {
                return Err(format!("无法执行 openssl: {}", e).into());
            }
        }
    } else {
        println!("✅ 使用现有证书");
    }

    // 配置证书
    let cert_config = CertConfig::from_paths(cert_path, key_path)
        .with_domains(vec!["localhost".to_string()]);
    let cert_manager_config = CertManagerConfig::shared(cert_config);
    let cert_manager = CertificateManager::from_config(cert_manager_config)?;

    let mut router = Router::new();
    router.enable_grpc_only();
    router.enable_h2();

    router.add_grpc_unary("/hello.HelloService/Hello", HelloHandler);
    router.add_grpc_unary("/ping.PingService/Ping", PingHandler);

    println!();
    println!("📡 gRPC 服务:");
    println!("   /hello.HelloService/Hello");
    println!("   /ping.PingService/Ping");
    println!();
    println!("按 Ctrl+C 停止");
    println!();

    let engine = RatEngine::builder()
        .worker_threads(4)
        .enable_logger()
        .router(router)
        .certificate_manager(cert_manager)
        .build()?;

    engine.start("0.0.0.0".to_string(), 50051).await?;

    Ok(())
}
