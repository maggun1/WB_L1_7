use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tokio::task;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

// Функция для остановки потока с использованием канала
fn stop_thread_with_channel() {
    println!("--- Остановка потока с использованием канала ---");

    let (tx, rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        loop {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(_) => {
                    println!("Поток получил сообщение...");
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    println!("Поток работает...");
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    println!("Канал закрыт, поток завершает работу.");
                    break;
                }
            }
        }
    });

    thread::sleep(Duration::from_secs(2));
    tx.send(()).unwrap();
    thread::sleep(Duration::from_secs(2));

    drop(tx);
    handle.join().unwrap();
}

// Функция для остановки задачи Tokio с использованием канала
async fn stop_tokio_task_with_channel() {
    println!("--- Остановка задач Tokio с использованием канала ---");

    let (tx, mut rx) = tokio::sync::mpsc::channel(32);

    // Запускаем задачу
    let task_handle = task::spawn(async move {
        loop {
            tokio::select! {
                // Ждем сообщения с тайм-аутом
                result = tokio::time::timeout(Duration::from_secs(1), rx.recv()) => {
                    match result {
                        Ok(Some(_)) => {
                            println!("Задача получила сообщение...");
                        }
                        Ok(None) => {
                            println!("Канал закрыт, задача завершает работу.");
                            break;
                        }
                        Err(_) => {
                            println!("Задача продолжает работу...");
                        }
                    }
                }
            }
        }

    });

    sleep(Duration::from_secs(2)).await;
    tx.send(()).await.unwrap();
    sleep(Duration::from_secs(2)).await;
    drop(tx);
    task_handle.await.unwrap();
}

// Функция для остановки задачи Tokio с использованием CancellationToken
async fn stop_tokio_task_with_cancellation_token() {
    println!("--- Остановка задач Tokio с использованием CancellationToken ---");

    let cancel_token = CancellationToken::new();
    let child_token = cancel_token.child_token();

    let token_task_handle = task::spawn(async move {
        loop {
            tokio::select! {
                _ = child_token.cancelled() => {
                    println!("Задача с CancellationToken завершает работу.");
                    break;
                }
                _ = sleep(Duration::from_secs(1)) => {
                    println!("Задача с CancellationToken работает...");
                }
            }
        }
    });

    sleep(Duration::from_secs(2)).await;

    cancel_token.cancel();

    token_task_handle.await.unwrap();
}

#[tokio::main]
async fn main() {
    stop_thread_with_channel();

    stop_tokio_task_with_channel().await;

    stop_tokio_task_with_cancellation_token().await;

    println!("Все задачи завершены.");
}
