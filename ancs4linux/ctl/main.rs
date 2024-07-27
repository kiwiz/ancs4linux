use structopt::StructOpt;
use serde_json::json;
use crate::common::apis::AdvertisingAPI;

#[derive(StructOpt, Debug)]
#[structopt(name = "ancs4linux")]
struct Opt {
    #[structopt(long, default_value = "ancs4linux.Advertising", help = "Advertising service path")]
    advertising_dbus: String,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    let advertising_api = AdvertisingAPI::connect(&opt.advertising_dbus).await;

    match opt.subcommand {
        SubCommand::GetAllHci => get_all_hci(advertising_api).await,
        SubCommand::EnableAdvertising { hci_address, name } => enable_advertising(advertising_api, hci_address, name).await,
        SubCommand::DisableAdvertising { hci_address } => disable_advertising(advertising_api, hci_address).await,
        SubCommand::EnablePairing => enable_pairing(advertising_api).await,
        SubCommand::DisablePairing => disable_pairing(advertising_api).await,
    }
}

async fn get_all_hci(advertising_api: AdvertisingAPI) {
    let hci_addresses = advertising_api.get_all_hci().await;
    println!("{}", json!(hci_addresses));
}

async fn enable_advertising(advertising_api: AdvertisingAPI, hci_address: String, name: String) {
    advertising_api.enable_advertising(hci_address, name).await;
}

async fn disable_advertising(advertising_api: AdvertisingAPI, hci_address: String) {
    advertising_api.disable_advertising(hci_address).await;
}

async fn enable_pairing(advertising_api: AdvertisingAPI) {
    advertising_api.enable_pairing().await;
}

async fn disable_pairing(advertising_api: AdvertisingAPI) {
    advertising_api.disable_pairing().await;
}
