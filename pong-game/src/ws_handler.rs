use bevy::prelude::*;
use futures_util::StreamExt;
use std::{fs::File, io::BufReader, sync::Arc};

use async_tungstenite::tokio::accept_async;
use async_tungstenite::tungstenite::Message;
use rustls::ServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use tokio::net::TcpListener;
use tokio_rustls::{TlsAcceptor, rustls};

use crate::{RacketCommandQueue, RacketTransformCommand, WsRuntime};
use anyhow::{Context, Result};

pub fn start_websocket_server(rt: Res<WsRuntime>, command_queue: Res<RacketCommandQueue>) {
    let command_queue = command_queue.clone();
    rt.0.spawn(async move {
        // 1. 加载证书与私钥
        let certs = load_certs("server.crt").unwrap();
        let key = load_key("server.key").unwrap();

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .context("构建 TLS 配置失败")
            .unwrap();
        let acceptor = TlsAcceptor::from(Arc::new(config));

        // 2. 启动 TCP 监听
        let listener = TcpListener::bind("0.0.0.0:8080")
            .await
            .context("无法绑定端口 8080")
            .unwrap();
        println!("✅ WSS 服务器已启动，监听 8080 端口");

        // 3. 接收连接循环
        loop {
            let (stream, addr) = listener.accept().await.unwrap();
            let acceptor = acceptor.clone();

            let command_queue = command_queue.clone();

            tokio::spawn(async move {
                match acceptor.accept(stream).await {
                    Ok(tls_stream) => {
                        println!("🔐 已建立 TLS 连接: {:?}", addr);
                        // TODO: 在这里继续处理 WebSocket 握手逻辑
                        let mut ws_stream = match accept_async(tls_stream).await {
                            Ok(ws_stream) => ws_stream,
                            Err(e) => {
                                eprintln!("❌ WebSocket 握手失败: {}", e);
                                return;
                            }
                        };
                        println!("🔗 WebSocket 握手成功: {:?}", addr);
                        while let Some(msg) = ws_stream.next().await {
                            match msg {
                                Ok(Message::Text(text)) => {
                                    // println!("收到文本: {}", text);
                                    let reply = text.to_uppercase();
                                    if ws_stream.send(Message::Text(reply.into())).await.is_err() {
                                        break;
                                    }
                                    // 操作 racket
                                    if let Some(command) = parse_transform_command(&text) {
                                        let mut queue = command_queue.0.lock().unwrap();
                                        queue.push(command);
                                        // println!("队列长度: {}", queue.len());
                                    }
                                }
                                Ok(Message::Close(_)) => {
                                    println!("🚪 连接关闭");
                                    break;
                                }
                                Err(e) => {
                                    eprintln!("接收消息出错: {}", e);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ TLS 握手失败: {}", e);
                    }
                }
            });
        }
    });
}

// 加载 X.509 PEM 格式证书
fn load_certs(path: &str) -> Result<Vec<CertificateDer<'static>>> {
    let certfile = File::open(path).context("无法打开证书文件")?;
    let mut reader = BufReader::new(certfile);
    certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .context("读取证书失败")
}

// 加载私钥，支持 PKCS8 和 RSA
fn load_key(path: &str) -> Result<PrivateKeyDer<'static>> {
    let keyfile = File::open(path).context("无法打开私钥文件")?;
    let mut reader = BufReader::new(keyfile);

    // 尝试 PKCS8 格式
    let mut keys = pkcs8_private_keys(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .context("读取 PKCS8 私钥失败")?;
    if let Some(key) = keys.into_iter().next() {
        return Ok(key.into());
    }

    // 尝试 RSA 格式
    let keyfile = File::open(path).context("无法重新打开私钥文件")?;
    let mut reader = BufReader::new(keyfile);
    let mut keys = rsa_private_keys(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .context("读取 RSA 私钥失败")?;

    keys.into_iter()
        .next()
        .map(Into::into)
        .ok_or_else(|| anyhow::anyhow!("未找到有效的私钥"))
}

use crate::CommandDataType;

fn parse_transform_command(text: &str) -> Option<RacketTransformCommand> {
    // 简单解析：x,y,z;rx,ry,rz,rw
    // 新格式: rotation:rx,ry,rz,rw
    //        position:dx,dy,dz
    let parts: Vec<&str> = text.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    match parts[0] {
        "rotation" => {
            let rot_vals: Vec<f32> = parts[1].split(',').filter_map(|s| s.parse().ok()).collect();
            if rot_vals.len() != 4 {
                return None;
            }
            return Some(RacketTransformCommand {
                command: CommandDataType::Rotation(Quat::from_xyzw(
                    rot_vals[0],
                    rot_vals[1],
                    rot_vals[2],
                    rot_vals[3],
                )),
            });
        }
        "position" => {
            let pos_vals: Vec<f32> = parts[1].split(',').filter_map(|s| s.parse().ok()).collect();
            if pos_vals.len() != 3 {
                return None;
            }
            return Some(RacketTransformCommand {
                command: CommandDataType::Position(Vec3::new(
                    pos_vals[0],
                    pos_vals[1],
                    pos_vals[2],
                )),
            });
        }
        _ => return None,
    }
}
