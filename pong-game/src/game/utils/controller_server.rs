use crate::game::utils::WsRuntime;
use bevy::prelude::*;

use std::env;

pub fn start_controller_server(rt: Res<WsRuntime>) {
    rt.0.spawn(async move {
        let cert_url = env::var("SSL_CERT_PATH").expect("CERT_URL 环境变量未设置");
        let key_url = env::var("SSL_KEY_PATH").expect("KEY_URL 环境变量未设置");
        let controller_route = warp::fs::dir("./dist");

        warp::serve(controller_route)
            .tls()
            .cert_path(cert_url)
            .key_path(key_url)
            .run(([0, 0, 0, 0], 3000))
            .await;
    });
    println!("✅ 控制器服务器已启动，监听 3000 端口");
}
