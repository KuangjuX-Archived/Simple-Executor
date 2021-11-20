use std::{future::Future, pin::Pin, sync::{Arc, Mutex}, task::{Context, Poll, Waker}, thread};
use futures::future::BoxFuture;


pub struct TaskFuture {
    id: usize,
    shared_state: Arc<Mutex<SharedState>>,
}

/// 任务的完成状态
pub struct SharedState {
    completed: bool,
    waker: Option<Waker>
}

impl Future for TaskFuture {
    type Output = usize;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            // 任务已经完成，返回 Ready
            Poll::Ready(self.id)
        }else{
            // 任务未完成，返回 Pending
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl TaskFuture {
    /// 注册 TaskFuture
    pub fn new<F>(job: F, id: usize) -> Self 
    where F: FnOnce() + Send + 'static {
        let shared_state = Arc::new(Mutex::new(SharedState{
            completed: false,
            waker: None
        }));
        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            job();
            let mut shared_state = thread_shared_state.lock().unwrap();
            shared_state.completed = true; 
            if let Some(waker) = shared_state.waker.take() {
                waker.wake()
            }
        });
        TaskFuture{
            id,
            shared_state
        }
    }
}

pub struct Task {
    pub future: Mutex<Option<BoxFuture<'static, ()>>>,
}

