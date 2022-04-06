use core::{future::Future, pin::Pin, task::{Context, Poll}};
use core::sync::atomic::{AtomicU64, Ordering};
use alloc::boxed::Box;
use futures_util::task::AtomicWaker;
use lazy_static::lazy_static;
use crate::task::executor::Executor;

pub mod simple_executor;
pub mod keyboard;
pub mod executor;


#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Debug)]
struct TaskId(u64);
impl TaskId{
    pub fn new()->Self{
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct Task{
    future: Pin<Box<dyn Future<Output= ()>>>,
    id: TaskId,
}
impl Task{
    pub fn new(future: impl Future<Output=()> + 'static) ->Task{
        Task{
            future: Box::pin(future),
            id: TaskId::new(),
        }
    }
    pub fn poll(&mut self, context: &mut Context) -> Poll<()>{
        self.future.as_mut().poll(context)
    }
}