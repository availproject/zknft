pub struct Proof {
    proof: String,
    receipts: Vec<Vec<u8>>,
}

pub struct Store {
    pub non_aggregated_nft_proofs: Vec<Proof>,
    pub non_aggregated_payment_proofs: Vec<Proof>,
}

pub enum ChainName {
    NFT,
    Payment,
}

impl Store {
    pub fn add_proof(&mut self, proof: Proof, chain: ChainName) {
        match chain {
            ChainName::NFT => self.non_aggregated_nft_proofs.push(proof),
            ChainName::Payment => self.non_aggregated_payment_proofs.push(proof),
        };
    }

    pub fn get_proofs(&self, chain: ChainName) -> &Vec<Proof> {
        match chain {
            ChainName::NFT => &self.non_aggregated_nft_proofs,
            ChainName::Payment => &self.non_aggregated_payment_proofs,
        }
    }

    pub fn clear_proofs(&mut self, chain: ChainName) {
        match chain {
            ChainName::NFT => &self.non_aggregated_nft_proofs.clear(),
            ChainName::Payment => &self.non_aggregated_payment_proofs.clear(),
        };
    }
}
