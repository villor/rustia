use futures::Future;
use tokio::runtime::Runtime;

pub fn start<F: Future + Send + 'static>(future: F) {
    std::thread::Builder::new()
        .name(String::from("tokio_runtime"))
        .spawn(move || {
            let rt = Runtime::new().expect("failed to create tokio runtime");
            rt.block_on(future);
        })
        .expect("failed to spawn tokio runtime thread");
}
