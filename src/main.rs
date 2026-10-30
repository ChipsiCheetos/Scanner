use port_scan::run_scanner;

mod cli;

#[tokio::main]
async fn main() {
    let args = cli::parse_args();

    println!("=== ЗАПУСК СКАНЕРА ===");
    println!("Цель: {}", args.target);
    println!("Диапазон портов: {}..={}", args.start_port, args.end_port);
    println!("Потоков (семафор): {}", args.concurrency);
    println!("Таймаут: {} мс", args.timeout);
    println!("======================");

    let range = args.start_port..=args.end_port;
    let open_ports = run_scanner(
        args.target,
        range, 
        args.concurrency, 
        args.timeout
    ).await;
    
    println!("\nСканирование завершено!");
    println!("Всего открытых портов найдено: {}", open_ports.len());
    println!("Список: {:?}", open_ports);
}