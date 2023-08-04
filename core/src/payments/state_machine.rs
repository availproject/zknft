use crate::{
  errors::Error,
  payments::types::{CallType, Account, CallParams as PaymentsCallParams, Address},
  state::State,
  traits::{Leaf, StateMachine},
  types::StateUpdate,
};
use primitive_types::U256;
use risc0_zkvm::sha::rust_crypto::{Digest as _};




pub struct PaymentsStateMachine {
  pub state: State<Account>,
}

impl StateMachine<Account> for PaymentsStateMachine {
  type CallParams = PaymentsCallParams;

  fn new() -> Self {
      let mut address_in_bytes = [0u8; 32];
      let mut address2_in_bytes = [0u8; 32];

      U256::from_dec_str("1").unwrap().to_big_endian(&mut address_in_bytes);
      U256::from_dec_str("2").unwrap().to_big_endian(&mut address2_in_bytes);

      let account1 = Account {
          address: Address(address_in_bytes),
          balance: 1000,
      };
      let account2 = Account {
          address: Address(address2_in_bytes),
          balance: 1000,
      };

      let mut state = State::new();

      state
          .update_set(vec![account1, account2])
          .expect("Init state failed.");

          PaymentsStateMachine { state }
  }

  fn call(&mut self, params: PaymentsCallParams) -> Result<StateUpdate<Account>, Error> {
      match params.call_type {
          CallType::Transfer => self.transfer(params),
          CallType::Mint => self.mint(params),
      }
  }

  fn load() -> Self {
      unimplemented!()
  }
}

impl PaymentsStateMachine {
  fn transfer(&mut self, params: PaymentsCallParams) -> Result<StateUpdate<Account>, Error> {
      let from_address_key = params.from.get_key();
      let to_address_key = params.to.get_key();

      let mut from_account: Account = match self.state.get(&from_address_key) {
        Ok(Some(i)) => i,
        Err(_e) => panic!("Error in finding account details"),
        Ok(None) => panic!("Account has no balance"),
      };

      if from_account.balance < params.amount {
          panic!("Not enough balance");
      }

      from_account.balance -= params.amount;

      let mut to_account = match self.state.get(&to_address_key) {
        Ok(Some(i)) => i,
        Err(_e) => panic!("Error in finding account details"),
        Ok(None) => Account {
          address: params.to, 
          balance: 0
        },
      };

      to_account.balance += params.amount; 

      self.state.update_set(vec![from_account, to_account])
  }

  fn mint(&mut self, params: PaymentsCallParams) -> Result<StateUpdate<Account>, Error> {
      let to_address_key = params.to.get_key();

      //TODO: Some check to ensure admin is minting. 

      let mut to_account = match self.state.get(&to_address_key) {
        Ok(Some(i)) => i,
        Err(_e) => panic!("Error in finding account details"),
        Ok(None) => Account {
          address: params.to, 
          balance: 0
        },
      };

      to_account.balance += params.amount; 

      self.state.update_set(vec![to_account])
  }
}
