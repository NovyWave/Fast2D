use moon::*;

async fn frontend() -> Frontend {
    Frontend::new()
        .title("Fast2D Blade Browser Example")
        .append_to_head(
            r#"
            <style>
                html, body {
                    margin: 0;
                    padding: 0;
                    height: 100%;
                    background: black;
                    color: white;
                    font-family: Arial, sans-serif;
                }
                #blade-canvas {
                    display: block;
                    width: 100%;
                    height: 100%;
                }
                .webgpu-warning {
                    position: absolute;
                    top: 50%;
                    left: 50%;
                    transform: translate(-50%, -50%);
                    text-align: center;
                    background: rgba(255, 0, 0, 0.8);
                    padding: 20px;
                    border-radius: 10px;
                    z-index: 1000;
                }
            </style>
            "#,
        )
        .append_to_head(
            r#"
            <script>
                // Check WebGPU support
                if (!navigator.gpu) {
                    document.addEventListener('DOMContentLoaded', function() {
                        const warning = document.createElement('div');
                        warning.className = 'webgpu-warning';
                        warning.innerHTML = `
                            <h2>WebGPU Not Supported</h2>
                            <p>This demo requires WebGPU support.</p>
                            <p>Please use:</p>
                            <ul style="text-align: left;">
                                <li>Chrome 113+ (stable)</li>
                                <li>Firefox 113+ (enable webgpu in about:config)</li>
                                <li>Safari 17+ (enable WebGPU in settings)</li>
                            </ul>
                        `;
                        document.body.appendChild(warning);
                    });
                }
            </script>
            "#,
        )
}

async fn up_msg_handler(_: UpMsgRequest<shared::AppConfig>) {
    // Handle frontend messages if needed
}

#[moon::main]
async fn main() -> std::io::Result<()> {
    start(frontend, up_msg_handler, |_| {}).await
}