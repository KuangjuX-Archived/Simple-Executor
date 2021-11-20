use simple_executor::{ TaskFuture, Executor } ;
use std::thread;
use std::time::Duration;
fn main() {
    let executor = Executor::new(10000);
    for i in 0..100 {
        let task_id = i;
        executor.add_task(async move {
            println!("Hello");
            TaskFuture::new(move || {
                println!("Hello Task {}", task_id);
                thread::sleep(Duration::new(1, task_id as u32));
            }, task_id).await;
            println!("Task {} done", task_id);
        })
    }
    executor.run();
}