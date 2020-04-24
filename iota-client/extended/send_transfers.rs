use anyhow::Result;
use bee_bundle::{Address, Hash, Transaction};
use bee_crypto::Kerl;
use bee_signing::IotaSeed;

use crate::response::{Input, Transfer};
use crate::Client;

/// Builder to construct SendTransfers API
//#[derive(Debug)]
pub struct SendTransfersBuilder<'a> {
    client: &'a Client<'a>,
    seed: Option<&'a IotaSeed<Kerl>>,
    transfers: Vec<Transfer>,
    security: u8,
    inputs: Option<Vec<Input>>,
    remainder: Option<Address>,
    depth: u8,
    min_weight_magnitude: u8,
    reference: Option<Hash>,
}

impl<'a> SendTransfersBuilder<'a> {
    pub(crate) fn new(client: &'a Client<'a>) -> Self {
        Self {
            client,
            seed: None,
            transfers: Default::default(),
            security: 2,
            inputs: None,
            remainder: None,
            depth: 3,
            min_weight_magnitude: 14,
            reference: Default::default(),
        }
    }

    /// Add iota seed
    pub fn seed(mut self, seed: &'a IotaSeed<Kerl>) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Add transfers
    pub fn transfers(mut self, transfers: Vec<Transfer>) -> Self {
        self.transfers = transfers;
        self
    }

    /// Set security level
    pub fn security(mut self, security: u8) -> Self {
        self.security = security;
        self
    }

    /// Add custom inputs. It is always better to provide inputs yourself
    /// since it will have to seaching valid inputs from the beginning.
    pub fn inputs(mut self, inputs: Vec<Input>) -> Self {
        self.inputs = Some(inputs);
        self
    }

    /// Add custom remainder
    pub fn remainder(mut self, remainder: Address) -> Self {
        self.remainder = Some(remainder);
        self
    }

    /// Set MWM depth
    pub fn depth(mut self, depth: u8) -> Self {
        self.depth = depth;
        self
    }

    /// Set difficulty of PoW
    pub fn min_weight_magnitude(mut self, min_weight_magnitude: u8) -> Self {
        self.min_weight_magnitude = min_weight_magnitude;
        self
    }

    /// Add reference hash
    pub fn reference(mut self, reference: Hash) -> Self {
        self.reference = Some(reference);
        self
    }

    /// Send SendTransfers request
    pub async fn send(self) -> Result<Vec<Transaction>> {
        let seed = match self.seed {
            Some(s) => s,
            None => return Err(anyhow!("Seed is not provided")),
        };

        let mut transfer = self
            .client
            .prepare_transfers()
            .seed(seed)
            .transfers(self.transfers)
            .security(self.security);

        if let Some(inputs) = self.inputs {
            transfer = transfer.inputs(inputs);
        }

        if let Some(remainder) = self.remainder {
            transfer = transfer.remainder(remainder);
        }

        let mut trytes: Vec<Transaction> = transfer.build().await?.into_iter().map(|x| x).collect();
        trytes.reverse();
        let mut send_trytes = self
            .client
            .send_trytes()
            .trytes(trytes)
            .depth(self.depth)
            .min_weight_magnitude(self.min_weight_magnitude);

        if let Some(reference) = self.reference {
            send_trytes = send_trytes.reference(reference);
        }

        Ok(send_trytes.send().await?)
    }
}
