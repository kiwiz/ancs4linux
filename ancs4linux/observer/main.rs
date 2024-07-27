use structopt::StructOpt;
use tokio::runtime::Runtime;
use log::info;

use ancs4linux::common::dbus::{SystemBus, EventLoop};
use ancs4linux::observer::scanner::Scanner;
use ancs4linux::observer::server::ObserverServer;

#[derive(StructOpt, Debug)]
#[structopt(name = "observer")]
struct Opt {
    #[structopt(long, default_value = "ancs4linux.Observer")]
    observer_dbus: String,
}

fn main() {
    let opt = Opt::from_args();
    env_logger::init();

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let server = ObserverServer::new();
        let scanner = Scanner::new(server.clone());
        server.set_scanner(scanner.clone());
        server.register().await;
        SystemBus::register_service(&opt.observer_dbus).await.unwrap();

        info!("Observing devices...");
        scanner.start_observing().await;
        EventLoop::run().await;
    });
}
