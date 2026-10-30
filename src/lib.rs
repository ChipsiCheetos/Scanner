use tokio::sync::mpsc;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub async fn run_scanner(
    target: std::net::IpAddr,
    port_range: std::ops::RangeInclusive<u16>,
    concurrency: usize,
    timeout_ms: u64,
) -> Vec<u16> {
    let cooldown = Duration::from_millis(timeout_ms);
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let (tx, mut rx) = mpsc::channel::<u16>(100);

    for port in port_range {
        let addrs = SocketAddr::new(target, port);
        let tx_clone = tx.clone();
        let Ok(permit) = semaphore.clone().acquire_owned().await else { break; };
        
        tokio::spawn(async move {
            let _held_permit = permit;

            if timeout(cooldown, TcpStream::connect(&addrs))
                .await
                .is_ok_and(|res| res.is_ok())
            {    
                let _ = tx_clone.send(port).await;
            }
        });  
    }
    drop(tx);

    let mut open_ports = vec![];
    while let Some(port) = rx.recv().await {
        println!("Нашли открытый порт: {port}");
        open_ports.push(port);
    }

    open_ports.sort();
    open_ports
}


#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use rand::seq::SliceRandom;
    use rand::rng;
    use std::net::IpAddr;
    use log::info;

    #[tokio::test]
    async fn test_main() {
        let _ = env_logger::builder().is_test(true).try_init();
        let mut test_ports: Vec<u16> = (8000..=8100).collect();
        let mut rng = rng();
        test_ports.shuffle(&mut rng);
        test_ports.truncate(20);
        let mut background_tasks = vec![];

        for &port in &test_ports {
            let addr = format!("127.0.0.1:{}", port);
            let listener = TcpListener::bind(&addr).await.unwrap();
            let handle = tokio::spawn(async move {
                while let Ok((_stream, _addr)) = listener.accept().await {
                    drop(_stream);
                }
            });
            background_tasks.push(handle);
        }
        
        tokio::task::yield_now().await;

        let localhost_ip: IpAddr = "127.0.0.1".parse().unwrap();
        let range = 8000..=8100;
        let concurrency = 200;
        let timeout_ms = 100;


        let mut found_ports = run_scanner(localhost_ip, range, concurrency, timeout_ms).await;

        test_ports.sort();
        found_ports.sort();

        assert_eq!(
            found_ports, test_ports,
            "Сканер нашёл не те порты которые были открыты!"
        );

        for handle in background_tasks {
            handle.abort();
        }

        info!("Тест успешно пройден! Сгенерировано портов: {}, Сканер нашел: {:?}", test_ports.len(), found_ports);
    }
}