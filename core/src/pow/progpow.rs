extern crate randomx;

use std::marker::PhantomData;

use crate::pow::common::EdgeType;
use crate::pow::error::{Error, ErrorKind};
use crate::pow::{PoWContext, Proof};
use crate::util::RwLock;
use crate::core::block::BlockHeader;

use keccak_hash::keccak_256;

use progpow::types::PpCompute;
use progpow::hardware::cpu::PpCPU;

lazy_static! {
	pub static ref PP_CPU: RwLock<PpCPU> = RwLock::new(PpCPU::new());
}

pub fn new_progpow_ctx<T>() -> Result<Box<dyn PoWContext<T>>, Error>
where
	T: EdgeType + 'static,
{
	Ok(Box::new(ProgPowContext {
        nonce: 0,
		height: 0,
		header: vec![],
		phantom: PhantomData,
	}))
}

pub struct ProgPowContext<T>
where
	T: EdgeType,
{
	pub header: Vec<u8>,
    pub nonce: u64,
	pub height: u64,
	phantom: PhantomData<T>,
}

impl<T> ProgPowContext<T>
where
	T: EdgeType,
{
	// make hash keccak256 with header pre pow
	fn header_hash(&self) -> [u8; 32] {
		// slice header
		let sheader = &self.header[0..(self.header.len()-8)];

		// copy header
		let cheader = sheader.to_vec();

		let mut header = [0u8; 32];
		keccak_256(&cheader, &mut header);

		header
	}
}

impl<T> PoWContext<T> for ProgPowContext<T>
where
	T: EdgeType,
{
	fn set_header_nonce(
		&mut self,
		header: Vec<u8>,
		nonce: Option<u64>,
		height: Option<u64>,
		_solve: bool,
	) -> Result<(), Error> {
		self.header = header;
		self.nonce = nonce.unwrap_or(0);
		self.height = height.unwrap_or(0);
		Ok(())
	}

	fn pow_solve(&mut self) -> Result<Vec<Proof>, Error> {
		let (_, m) = {
			let progpow = PP_CPU.read();
			progpow.verify(
				&self.header_hash(), self.height, self.nonce).unwrap()
		};

		let mix: [u8; 32] = unsafe { ::std::mem::transmute(m) };

		Ok(vec![Proof::ProgPowProof { mix }])
	}

	fn verify(&mut self, proof: &Proof) -> Result<(), Error> {
		let (_, tm) = {
			let progpow = PP_CPU.read();
			progpow.verify(
				&self.header_hash(), self.height, self.nonce).unwrap()
		};

		if let Proof::ProgPowProof { ref mix } = proof {
			let mix_test: [u32; 8] = unsafe { ::std::mem::transmute(*mix) };
			if mix_test == tm {
                return Ok(());
            }
		}

		Err(ErrorKind::Verification("Hash progpow invalid!".to_string()))?
	}
}
