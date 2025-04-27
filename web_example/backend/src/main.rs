use moon::*;

async fn frontend() -> Frontend {
    Frontend::new()
        .title("Fast2D Web Example")
        .index_by_robots(false)
}

async fn up_msg_handler(_: UpMsgRequest<()>) {}

#[moon::main]
async fn main() -> std::io::Result<()> {
    start(frontend, up_msg_handler, |_| {}).await
}
