use futures::future::FutureExt;
use std::{future::Future, sync::{Arc, Mutex}, task::Context};
use crossbeam_queue::ArrayQueue;
use std::task::{ Wake, Waker };

use super::Task;

/// 执行器从 channel 中接受任务并且运行它们
pub struct Executor {
    ready_queue: Arc<ArrayQueue<Arc<Task>>>,
}


impl Executor {
    pub fn new(capacity: usize) -> Self {
        Self {
            ready_queue: Arc::new(ArrayQueue::new(capacity))
        }
    }

    /// 向执行器中添加任务
    pub fn add_task(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(
            Task {
                future: Mutex::new(Some(future)),
            }
        );
        if let Err(_) = self.ready_queue.push(task) {
            panic!("Fail to wake");
        }
    }

    /// 启动执行器
    pub fn run(&self) {
       loop {
           if let Some(task) = self.ready_queue.pop() {
               let mut future_slot = task.future.lock().unwrap();
               if let Some(mut future) = future_slot.take() {
                   let waker = TaskWaker::new(Arc::clone(&task), Arc::clone(&self.ready_queue));
                   let context = &mut Context::from_waker(&waker);
                   if future.as_mut().poll(context).is_pending() {
                       *future_slot = Some(future);
                   }
               }
           }
       }
    }
}

pub struct TaskWaker{
    task: Arc<Task>,
    queue: Arc<ArrayQueue<Arc<Task>>>
}

impl TaskWaker {
    pub fn new(task: Arc<Task>, queue: Arc<ArrayQueue<Arc<Task>>>) -> Waker {
        Waker::from(Arc::new(TaskWaker{
            task,
            queue
        }))
    }

    pub fn wake_task(&self) {
        if let Err(_) = self.queue.push(Arc::clone(&self.task)) {
            panic!("Fail to wake");
        }
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}

