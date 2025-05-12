use crate::WsRuntime;
use bevy::prelude::*;
use warp::Filter;

pub fn start_controller_server(rt: Res<WsRuntime>) {
    rt.0.spawn(async move {
        let controller_route = warp::fs::dir("./dist");

        warp::serve(controller_route)
            .tls()
            .cert_path("server.crt")
            .key_path("server.key")
            .run(([0, 0, 0, 0], 3000))
            .await;
    });
}
