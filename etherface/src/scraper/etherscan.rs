use crate::scraper::Scraper;
use anyhow::Error;
use chrono::Utc;
use etherface_lib::api::etherscan::EtherscanClient;
use etherface_lib::database::handler::DatabaseClient;
use etherface_lib::model::MappingSignatureEtherscan;
use etherface_lib::parser;

#[derive(Debug)]
pub struct EtherscanScraper;
impl Scraper for EtherscanScraper {
    fn start(&self) -> Result<(), Error> {
        let dbc = DatabaseClient::new()?;
        let esc = EtherscanClient::new()?;

        loop {
            // Scrape signatures from unvisited contracts
            for contract in dbc.etherscan_contract().get_unvisited() {
                if let Ok(abi_content) = esc.get_abi(&contract.address) {
                    // Insert all scraped signatures
                    for signature in parser::from_abi(&abi_content)? {
                        let inserted_signature = dbc.signature().insert(&signature);

                        let mapping = MappingSignatureEtherscan {
                            signature_id: inserted_signature.id,
                            contract_id: contract.id,
                            kind: signature.kind,
                            visibility: signature.visibility,
                            added_at: Utc::now(),
                        };

                        dbc.mapping_signature_etherscan().insert(&mapping);
                    }

                    dbc.etherscan_contract().set_visited(&contract);
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(5 * 60));
        }
    }
}
