use structopt::StructOpt;
use tokio::runtime::Runtime;
use log::info;

use crate::advertising::advertisement::AdvertisingManager;
use crate::advertising::pairing::PairingManager;
use crate::advertising::server::AdvertisingServer;
use crate::common::dbus::SystemBus;

#[derive(StructOpt, Debug)]
#[structopt(name = "ancs4linux")]
struct Opt {
    #[structopt(long, default_value = "ancs4linux.Advertising", help = "Service path")]
    advertising_dbus: String,
}

fn main() {
    let opt = Opt::from_args();
    env_logger::init();

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let pairing_manager = PairingManager::new();
        let advertising_manager = AdvertisingManager::new(pairing_manager.clone());
        let server = AdvertisingServer::new(pairing_manager.clone(), advertising_manager.clone());

        pairing_manager.register(&server).await;
        advertising_manager.register().await;
        server.register().await;
        SystemBus::new().await.unwrap().register_service(&opt.advertising_dbus).await.unwrap();

        info!("Ready to advertise...");
    });
}
