use futures::{future::FutureExt, task::waker_ref};
use std::{future::Future, sync::mpsc::{sync_channel, Receiver, SyncSender}, sync::{Arc, Mutex}, task::Context};

use super::Task;

/// 执行器从 channel 中接受任务并且运行它们
pub struct Executor {
    ready_queue: Receiver<Arc<Task>>
}

/// `Spawner` 在 task channle 上产生新的 Futures
pub struct Spawner {
    pub id: usize,
    task_sender: SyncSender<Arc<Task>>
}

impl Spawner {
    /// 向生成器中推送任务
    pub fn spawn(&mut self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(
            Task{
                future: Mutex::new(Some(future)),
                task_sender: self.task_sender.clone()
            }
        );
        // 向 channel 中发送任务
        self.id += 1;
        self.task_sender.send(task).expect("too many tasks queued");
    }
}

impl Executor {
    /// 启动执行器
    pub fn run(&self) {
        // 从任务队列中接受任务
        while let Ok(task) = self.ready_queue.recv() {
            let mut future_slot = task.future.lock().unwrap();
            // 获取任务的 Future
            if let Some(mut future) = future_slot.take() {
                // 获取任务唤醒者
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&*waker);
                // 轮询任务查看是否完成
                if future.as_mut().poll(context).is_pending() {
                    // 由于future_slot的所有权已经移出来了，因此需要重新赋值
                    *future_slot = Some(future)
                }
            }
        }
    }
}

/// new 执行器和生成器，生成器负责推送任务，执行器负责执行任务
pub fn new_executor_and_spawner() -> (Executor, Spawner) {
    const MAX_QUEUED_TASKS: usize = 10000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
    (Executor{ ready_queue }, Spawner{ id: 0, task_sender})
}