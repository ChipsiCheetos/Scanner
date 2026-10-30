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
    let (tx, mut rx) = mpsc::unbounded_channel::<u16>();

    for port in port_range {
        let addrs = SocketAddr::new(target, port);
        let tx_clone = tx.clone();
        let Ok(permit) = semaphore.clone().acquire_owned().await else { continue; };
        
        tokio::spawn(async move {
            let _held_permit = permit;

            if timeout(cooldown, TcpStream::connect(&addrs))
                .await
                .is_ok_and(|res| res.is_ok())
            {    
                let _ = tx_clone.send(port);
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
    use std::net::IpAddr;
    use log::info;

    #[tokio::test]
    async fn test_main() {
        let _ = env_logger::builder().is_test(true).try_init();
        let mut test_ports: Vec<u16> = vec![];
        let mut background_tasks = vec![];

        for _ in 0..=20 {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

            let assigned_port = listener.local_addr().unwrap().port();
            test_ports.push(assigned_port);

            let handle = tokio::spawn(async move {
                while let Ok((_stream, _addr)) = listener.accept().await {
                    drop(_stream);
                }
            });
            background_tasks.push(handle);
        }
        
        tokio::task::yield_now().await;

        let localhost_ip: IpAddr = "127.0.0.1".parse().unwrap();
        let min_port = *test_ports.iter().min().unwrap();
        let max_port = *test_ports.iter().max().unwrap();
        let range = min_port..=max_port;
        let concurrency = 200;
        let timeout_ms = 100;


        let found_ports = run_scanner(localhost_ip, range, concurrency, timeout_ms).await;

        let mut filtered_found: Vec<u16> = found_ports
            .iter()
            .filter(|port| test_ports.contains(port))
            .copied()
            .collect();

        test_ports.sort();
        filtered_found.sort();

        assert_eq!(
            filtered_found, test_ports,
            "Сканер нашёл не те порты которые были открыты!"
        );

        for handle in background_tasks {
            handle.abort();
        }

        info!("Тест успешно пройден! Сгенерировано портов: {}, Сканер нашел: {:?}", test_ports.len(), found_ports);
    }
}