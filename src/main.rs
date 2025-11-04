use std::{fs, path::PathBuf, str::FromStr};

use lwk_common::{Signer};
use lwk_signer::{
    SwSigner,
    bip39::{Mnemonic},
};
use lwk_wollet::{
    ElementsNetwork, Wollet, WolletDescriptor, elements::Address, blocking::{BlockchainBackend, EsploraClient} 
};

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let network = ElementsNetwork::default_regtest();

    // Normally would be something like 'https://blockstream.info/liquid/api'
    let esplora_url = "http://localhost:30001/";
    let mut client = EsploraClient::new(esplora_url, network)?;


    let mnemonic = Mnemonic::generate(12)?;
    print!("\n\nSeed phrase: {}\n\n", mnemonic.to_string());
    let signer = SwSigner::new(&mnemonic.to_string(), network == ElementsNetwork::Liquid)?;

    let desc = signer.wpkh_slip77_descriptor()?;
    println!("Descriptor: {:?}\n", desc);
    // example descriptor: "ct(slip77(2b144be071eec6f552ebc6aedede00e9de3485c2a1d476902a6c1ba1dc76864f),elwpkh([bdf02761/84h/1h/0h]tpubDDaEJwLwsjPyr6RinWvX5P1mM5eHeRN5ojpVVN3WPk2fhY7Jcp3yaooyx78vGb8FN9MDG4ocFZfRsKrQP4j7Pgd5ahovt7AHMZQeS7Tv3JY/<0;1>/*))#m3rgynjg";

    let wd = WolletDescriptor::from_str(&desc)?;
    let db_root_dir = PathBuf::from("./wallet_dir");
    fs::create_dir_all(&db_root_dir).unwrap();
    
    let mut wollet_c = Wollet::with_fs_persist(network, wd, db_root_dir)?;

    // Create an address
    let address = wollet_c.address(Some(0))?;
    println!("Address: {:?}\n", address.address().to_string());

    println!("Press enter to continue\n");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input);

    if let Some(update) = client.full_scan(&wollet_c)? {
        wollet_c.apply_update(update)?;
    }

    // Get balance
    let balance = wollet_c.balance()?;
    println!("Balance: {:?}", balance);

    let recipient_address = Address::from_str("el1qq0yzpekxusrlk2jlzpzkqa8esg7lpa5yptek6c2rvxmugsperls7fd0z9tg50e65m98htg80k42c96vt7gy6skl040tseayjr")?;

    let mut pset = wollet_c.tx_builder().add_recipient(&recipient_address, 10000, network.policy_asset())?.disable_ct_discount().finish()?;

    let sigs_added = signer.sign(&mut pset);
    let tx = wollet_c.finalize(&mut pset)?;
    let txid = client.broadcast(&tx)?;
    println!("TXID: {:?}", txid);


    Ok(())
}
