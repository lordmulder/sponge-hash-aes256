// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use crossbeam_channel::SendError;
use std::{
    any::Any,
    iter::repeat_n,
    num::NonZeroUsize,
    thread::{self, available_parallelism, JoinHandle},
};

use crate::{
    arguments::Args,
    environment::{get_thread_count, InvalidValue},
};

/// Maximum number of threads
pub const MAX_THREADS: usize = 64usize;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Error type to signal that a task was cancelled
pub struct Cancelled;

impl<T> From<SendError<T>> for Cancelled {
    fn from(_: SendError<T>) -> Self {
        Self
    }
}

pub type TaskResult = Result<(), Cancelled>;

// ---------------------------------------------------------------------------
// Detect number of CPU cores
// ---------------------------------------------------------------------------

/// Map the number of available CPU cores to the number of threads
///
/// **Note:** This avoids running too many parallel threads on systems with a large number of CPU cores!
fn map_cores_to_threads(cores: NonZeroUsize) -> NonZeroUsize {
    let thread_count = (2.0 * (cores.get() as f64).log2()).floor() as usize;
    NonZeroUsize::new(thread_count.max(1usize)).unwrap()
}

/// Determine the number of threads
pub fn detect_thread_count(args: &Args) -> Result<NonZeroUsize, InvalidValue> {
    if args.multi_threading {
        match get_thread_count()?.map(|value| value.min(MAX_THREADS)).unwrap_or(usize::MIN) {
            usize::MIN => Ok(map_cores_to_threads(available_parallelism().unwrap_or(NonZeroUsize::MIN))),
            count => Ok(NonZeroUsize::new(count).unwrap()),
        }
    } else {
        Ok(NonZeroUsize::MIN)
    }
}

// ---------------------------------------------------------------------------
// Thread pool
// ---------------------------------------------------------------------------

/// Implements a simple thread pool to run `n` instances of a task in parallel
pub struct ThreadPool {
    thread_handles: Option<Vec<JoinHandle<TaskResult>>>,
}

impl ThreadPool {
    /// Start `n` parallel tasks (threads)
    pub fn new<T>(thread_count: NonZeroUsize, task_func: T) -> Self
    where
        T: FnOnce() -> TaskResult,
        T: Clone + Send + 'static,
    {
        debug_assert!(thread_count > NonZeroUsize::MIN);
        let mut thread_handles = Vec::with_capacity(thread_count.get());

        for task_instance in repeat_n(task_func, thread_count.get()) {
            thread_handles.push(thread::spawn(task_instance))
        }

        Self { thread_handles: Some(thread_handles) }
    }

    /// Wait until all running tasks (threads) have finished
    pub fn join(mut self) -> Result<TaskResult, Box<dyn Any + Send + 'static>> {
        let mut cancelled = false;

        for thread in self.thread_handles.take().unwrap().into_iter() {
            if let Err(Cancelled) = thread.join()? {
                cancelled = true
            }
        }

        if cancelled {
            Ok(Err(Cancelled))
        } else {
            Ok(Ok(()))
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        if self.thread_handles.as_ref().is_some() {
            panic!("Did not shut down the thread-pool before dropping!");
        }
    }
}
