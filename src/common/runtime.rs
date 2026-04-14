use std::sync::OnceLock;
use tokio::runtime::Runtime;

static RT: OnceLock<Runtime> = OnceLock::new();

/// 返回进程级共享的 tokio 多线程 Runtime，首次调用时初始化。
/// 替代每次点击都 Runtime::new() 的做法。
pub fn rt() -> &'static Runtime {
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .build()
            .expect("Failed to create tokio runtime")
    })
}
