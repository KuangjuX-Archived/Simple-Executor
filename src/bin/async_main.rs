use simple_executor::{ new_executor_and_spawner, TaskFuture };
use std::thread;
use std::time::Duration;
fn main() {
    let (executor, mut spanwer) = new_executor_and_spawner();
    spanwer.spawn(async {
        let id = TaskFuture::new(move || {
            println!("Hello 1");
            thread::sleep(Duration::new(1, 0));
        }, 1).await;
        println!("Task {} done!", id);
    });
    spanwer.spawn(async {
        let id = TaskFuture::new(move || {
            println!("Hello 2");
        }, 2).await;
        println!("Task {} done!", id);
    });
    drop(spanwer);
    executor.run();
}