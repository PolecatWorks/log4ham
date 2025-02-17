use tokio::runtime::{self, Runtime};

pub fn rt_multithreaded(threads: usize) -> Runtime {
    if threads == 0 {
        runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap()
    } else {
        runtime::Builder::new_multi_thread()
            .worker_threads(threads)
            .thread_name(format!("threads-{threads}"))
            .thread_stack_size(3 * 1024 * 1024)
            .enable_io()
            .enable_time()
            .build()
            .unwrap()
    }
}
