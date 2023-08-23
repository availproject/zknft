#[cfg(feature = "native")]
use avail_subxt::{
    api::runtime_types::{da_control::pallet::Call, da_runtime::RuntimeCall::DataAvailability},
    primitives::AppUncheckedExtrinsic,
};
use bytes::Bytes;
#[cfg(feature = "native")]
use codec::Encode;
use primitive_types::H256;
use serde::{Deserialize, Serialize};
use sov_rollup_interface::da::{BlobReaderTrait, CountedBufReader};

use super::address::AvailAddress;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
//pub struct AvailBlobTransaction(pub AppUncheckedExtrinsic);
pub struct AvailBlobTransaction {
    blob: Vec<u8>,
    hash: [u8; 32],
    address: AvailAddress,
    counted_buf_blob: CountedBufReader<Bytes>,
}

impl BlobReaderTrait for AvailBlobTransaction {
    type Data = Bytes;

    type Address = AvailAddress;

    fn sender(&self) -> AvailAddress {
        self.address.clone()
    }

    fn data(&self) -> &CountedBufReader<Self::Data> {
        &self.counted_buf_blob
    }

    fn data_mut(&mut self) -> &mut CountedBufReader<Self::Data> {
        &mut self.counted_buf_blob
    }

    fn hash(&self) -> [u8; 32] {
        self.hash
    }
}

#[cfg(feature = "native")]
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
            hash: H256::from(sp_core::blake2_256(&unchecked_extrinsic.encode())).to_fixed_bytes(),
            address,
            blob: blob.clone(),
            counted_buf_blob: CountedBufReader::<Bytes>::new(Bytes::copy_from_slice(&blob)),
        }
    }

    pub fn blob(&self) -> Vec<u8> {
        self.blob.clone()
    }
}
