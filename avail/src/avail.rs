#[cfg(not(feature = "verifier"))]
use avail_subxt::api::runtime_types::{
    da_control::pallet::Call, da_runtime::RuntimeCall::DataAvailability,
};
use avail_subxt::primitives::AppUncheckedExtrinsic;
use avail_subxt::primitives::Header as SubxtHeader;

#[cfg(not(feature = "verifier"))]
use codec::Encode;
use core::fmt::{Display, Formatter};
use primitive_types::H256;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Confidence {
    pub block: u32,
    pub confidence: f64,
    pub serialised_confidence: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExtrinsicsData {
    pub block: u32,
    pub extrinsics: Vec<AppUncheckedExtrinsic>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Eq, Hash)]
pub struct AvailAddress(pub [u8; 32]);

impl Display for AvailAddress {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        let hash = H256(self.0);
        write!(f, "{hash}")
    }
}

impl AsRef<[u8]> for AvailAddress {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<[u8; 32]> for AvailAddress {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

impl FromStr for AvailAddress {
    type Err = <H256 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let h_256 = H256::from_str(s)?;

        Ok(Self(h_256.to_fixed_bytes()))
    }
}

impl<'a> TryFrom<&'a [u8]> for AvailAddress {
    type Error = anyhow::Error;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self(<[u8; 32]>::try_from(value)?))
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub parent_hash: H256,
    pub number: u32,
    pub state_root: H256,
    pub extrinsics_root: H256,
    pub data_root: H256,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct AvailHeader {
    hash: H256,

    pub header: Header,
}

impl AvailHeader {
    pub fn new(header: SubxtHeader, hash: H256) -> Self {
        Self {
            hash,
            header: Header {
                parent_hash: header.parent_hash,
                number: header.number,
                state_root: header.state_root,
                data_root: header.data_root(),
                extrinsics_root: header.extrinsics_root,
            },
        }
    }

    fn hash(&self) -> H256 {
        self.hash
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]

pub struct AvailBlobTransaction {
    blob: Vec<u8>,
    hash: H256,
    address: AvailAddress,
}

impl AvailBlobTransaction {
    fn sender(&self) -> AvailAddress {
        self.address.clone()
    }

    fn hash(&self) -> H256 {
        self.hash
    }

    fn verified_data(&self) -> &[u8] {
        &self.blob
    }

    fn total_len(&self) -> usize {
        self.blob.len()
    }
}

impl AvailBlobTransaction {
    pub fn new(unchecked_extrinsic: &AppUncheckedExtrinsic) -> Self {
        let address = match &unchecked_extrinsic.signature {
            Some((subxt::utils::MultiAddress::Id(id), _, _)) => AvailAddress(id.clone().0),
            _ => unimplemented!(),
        };
        let blob = match &unchecked_extrinsic.function {
            DataAvailability(Call::submit_data { data }) => data.0.clone(),
            _ => unimplemented!(),
        };

        AvailBlobTransaction {
            hash: H256(sp_core_hashing::blake2_256(&unchecked_extrinsic.encode())),
            address,
            blob,
        }
    }

    pub fn combine_hash(&self, hash: H256) -> H256 {
        let mut combined_hashes: Vec<u8> = Vec::with_capacity(64);
        combined_hashes.extend_from_slice(hash.0.as_ref());
        combined_hashes.extend_from_slice(self.hash().0.as_ref());

        H256(sp_core_hashing::blake2_256(&combined_hashes))
    }

    pub fn blob(&self) -> &[u8] {
      &self.blob
  }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct AvailBlock {
    pub header: AvailHeader,
    pub transactions: Vec<AvailBlobTransaction>,
}

impl AvailBlock {
    pub fn hash(&self) -> H256 {
        self.header.hash()
    }

    pub fn header(&self) -> &AvailHeader {
        &self.header
    }

    pub fn find_tx(&self, hash: &H256) -> Option<AvailBlobTransaction> {
        for transaction in &self.transactions {
            if transaction.hash == *hash {
                return Some(transaction.clone());
            }
        }

        None
    }

    // Below not required at the moment.
    // fn validity_condition(&self) -> ChainValidityCondition {
    //     let mut txs_commitment: H256 = [0u8; 32];

    //     for tx in &self.transactions {
    //         txs_commitment = tx.combine_hash(txs_commitment);
    //     }

    //     ChainValidityCondition {
    //         prev_hash: *self.header().prev_hash().inner(),
    //         block_hash: <Self as SlotData>::hash(self),
    //         txs_commitment,
    //     }
    // }
}
