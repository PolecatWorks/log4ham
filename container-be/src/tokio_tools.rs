//! Module to handle easy sending functions to tokio
//!
//! The provides two functions one function run_in_tokio creates and sends the function to tokio.
//! The second function run_in_tokio_with_cancel allows the creation of a CancellationToken which can be used to shut down the tokio async.

use crate::error::MyError;
use futures::Future;
use log::{error, info};

use serde::Deserialize;
use tokio::runtime::{self, Runtime};
use tokio_util::sync::CancellationToken;

#[derive(Deserialize, Debug, Clone)]
pub struct ThreadRuntime {
    pub threads: usize,
    pub stack_size: usize,
}

impl Default for ThreadRuntime {
    fn default() -> Self {
        ThreadRuntime {
            threads: 0,
            stack_size: 3_000_000,
        }
    }
}

pub fn rt_multithreaded(name: &str, runtime: &ThreadRuntime) -> Result<Runtime, MyError> {
    if runtime.threads == 0 {
        runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(MyError::from)
    } else {
        runtime::Builder::new_multi_thread()
            .worker_threads(runtime.threads)
            .thread_name(name)
            .thread_stack_size(runtime.stack_size)
            .enable_io()
            .enable_time()
            .build()
            .map_err(MyError::from)
    }
}

/// run async function inside tokio instance on current thread
pub fn run_in_tokio<F, T>(name: &str, runtime: &ThreadRuntime, my_function: F) -> F::Output
where
    F: Future<Output = Result<T, MyError>>,
{
    info!("starting Tokio");

    rt_multithreaded(name, runtime)
        .expect("Runtime created")
        .block_on(my_function)
}

/// Run async with cancellability via CancellationToken
pub fn run_in_tokio_with_cancel<F, T>(
    name: &str,
    runtime: &ThreadRuntime,
    cancel: CancellationToken,
    my_function: F,
) -> F::Output
where
    F: Future<Output = Result<T, MyError>>,
{
    run_in_tokio(name, runtime, async {
        tokio::select! {
            _ = cancel.cancelled() => {
                error!("Token cancelled");
                Err(MyError::Cancelled)
            },
            z = my_function => {
                eprintln!("Completed function");
                z
            },
        }
    })
}
