use core::time::Duration;

use anyhow::anyhow;
use avail_subxt::api;
use avail_subxt::api::runtime_types::bounded_collections::bounded_vec::BoundedVec;
use avail_subxt::primitives::AvailExtrinsicParams;
use avail_subxt::AvailConfig;
use avail_subxt::config::Header;
use reqwest::StatusCode;
use sp_core::crypto::Pair as PairTrait;
use sp_keyring::sr25519::sr25519::Pair;
use subxt::tx::PairSigner;
use subxt::OnlineClient;
use primitive_types::H256;
use crate::avail::AvailBlobTransaction;
use crate::avail::AvailBlock;
use crate::avail::AvailHeader;
use crate::avail::{Confidence, ExtrinsicsData};

/// Runtime configuration for the DA service
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct DaServiceConfig {
    pub light_client_url: String,
    pub node_client_url: String,
    //TODO: Safer strategy to load seed so it is not accidentally revealed.
    pub seed: String,
    pub app_id: u32,
}

#[derive(Clone)]
pub struct DaProvider {
    pub node_client: OnlineClient<AvailConfig>,
    pub light_client_url: String,
    app_id: u32,
    signer: PairSigner<AvailConfig, Pair>,
}

enum HeightOrHash {
    Hash([u8; 32]), 
    Height(u64),
}

impl DaProvider {
    fn appdata_url(&self, block_num: u64) -> String {
        let light_client_url = self.light_client_url.clone();
        format!("{light_client_url}/v1/appdata/{block_num}")
    }

    fn confidence_url(&self, block_num: u64) -> String {
        let light_client_url = self.light_client_url.clone();
        format!("{light_client_url}/v1/confidence/{block_num}")
    }

    pub async fn new(config: DaServiceConfig) -> Self {
        let pair = Pair::from_string_with_seed(&config.seed, None).unwrap();
        let signer = PairSigner::<AvailConfig, Pair>::new(pair.0.clone());

        let node_client = avail_subxt::build_client(config.node_client_url.to_string(), false)
            .await
            .unwrap();
        let light_client_url = config.light_client_url;

        DaProvider {
            node_client,
            light_client_url,
            signer,
            app_id: config.app_id
        }
    }
}

const POLLING_TIMEOUT: Duration = Duration::from_secs(60);
const POLLING_INTERVAL: Duration = Duration::from_secs(2);

async fn wait_for_confidence(confidence_url: &str) -> anyhow::Result<()> {
    let start_time = std::time::Instant::now();

    loop {
        println!("Waiting for confidence: {:?}", &confidence_url);
        if start_time.elapsed() >= POLLING_TIMEOUT {
            return Err(anyhow!("Timeout..."));
        }

        let response = reqwest::get(confidence_url).await?;
        if response.status() != StatusCode::OK {
            println!("Confidence not received");
            tokio::time::sleep(POLLING_INTERVAL).await;
            continue;
        }

        let response: Confidence = serde_json::from_str(&response.text().await?)?;
        if response.confidence < 92.5 {
            println!("Confidence not reached");
            tokio::time::sleep(POLLING_INTERVAL).await;
            continue;
        }

        break;
    }

    Ok(())
}

async fn wait_for_appdata(appdata_url: &str, block: u32) -> anyhow::Result<ExtrinsicsData> {
    let start_time = std::time::Instant::now();

    loop {
        println!("Getting app data {:?}", appdata_url);

        if start_time.elapsed() >= POLLING_TIMEOUT {
            return Err(anyhow!("Timeout..."));
        }

        let response = reqwest::get(appdata_url).await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(ExtrinsicsData {
                block,
                extrinsics: vec![],
            });
        }
        if response.status() != StatusCode::OK {
            tokio::time::sleep(POLLING_INTERVAL).await;
            continue;
        }

        let appdata: ExtrinsicsData = serde_json::from_str(&response.text().await?)?;
        return Ok(appdata);
    }
}

impl DaProvider {
    // Make an RPC call to the node to get the finalized block at the given height, if one exists.
    // If no such block exists, block until one does.
    async fn get_finalized_at(&self, height_or_hash: HeightOrHash) -> Result<AvailBlock, anyhow::Error> {
        let node_client = self.node_client.clone();
        let (header, hash) = match height_or_hash {
            HeightOrHash::Height(i) => {
                let hash = match node_client
                .rpc()
                .block_hash(Some(i.into()))
                .await? {
                    Some(i) => i, 
                    None => return Err(anyhow!("Hash for height {} not found.", i))
                };
                
                let header = match node_client.rpc().header(Some(hash)).await? {
                    Some(i) => i, 
                    None => return Err(anyhow!("Header not found for hash: {}", hash))
                };

                (header, hash)
            }, 
            HeightOrHash::Hash(i) => {
                let hash = H256::from(i);

                let header = match node_client.rpc().header(Some(hash)).await? {
                    Some(i) => i, 
                    None => return Err(anyhow!("Header not found for hash: {}", hash))
                };
               (header, hash)
            }
        };
        
        let height = header.number();
        let confidence_url = self.confidence_url(height.into());
        let appdata_url = self.appdata_url(height.into());

        //TODO: Wait for confidence.
        //wait_for_confidence(&confidence_url).await?;
        let appdata = wait_for_appdata(&appdata_url, height).await?;

        let header = AvailHeader::new(header, hash);
        let transactions = appdata
            .extrinsics
            .iter()
            .map(AvailBlobTransaction::new)
            .collect();
        Ok(AvailBlock {
            header,
            transactions,
        })
    }

    // Make an RPC call to the node to get the block at the given height
    // If no such block exists, block until one does.
    pub async fn get_block_at(&self, height: u64) -> Result<AvailBlock, anyhow::Error> {
        self.get_finalized_at(HeightOrHash::Height(height)).await
    }

    pub async fn get_block_with_hash(&self, hash: [u8; 32]) -> Result<AvailBlock, anyhow::Error> {
        self.get_finalized_at(HeightOrHash::Hash(hash)).await
    }

    pub async fn send_transaction(&self, blob: &[u8]) -> Result<(H256, H256), anyhow::Error> {
        println!("Started submissions");

        let data_transfer = api::tx()
        .data_availability()
        .submit_data(BoundedVec(blob.to_vec()));
        
        let extrinsic_params = AvailExtrinsicParams::new_with_app_id(self.app_id.into());

        let tx_progress = self.node_client
        .tx()
        .sign_and_submit_then_watch(&data_transfer, &self.signer, extrinsic_params)
        .await?;

        let (block_hash, index) = tx_progress
        .wait_for_finalized_success()
        .await
        .map(|event| (
                event.block_hash(),
                event.extrinsic_hash(),
            )
        )?;

        Ok((block_hash, index))
    }
}
