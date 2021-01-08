use sp_core::{Pair, Public, sr25519};
use node_subtensor_runtime::{
	AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig,
	SudoConfig, SystemConfig, WASM_BINARY, Signature
};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{Verify, IdentifyAccount};
use sc_service::ChainType;
use sp_core::crypto::Ss58Codec;
use sc_service::config::MultiaddrWithPeerId;
use sp_runtime::sp_std::str::FromStr;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}


pub fn get_aura_from_ss58_addr(s: &str) -> AuraId {
	AuraId::from_ss58check(s).unwrap()
}

pub fn get_grandpa_from_ss58_addr(s: &str) -> GrandpaId {
	GrandpaId::from_ss58check(s).unwrap()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(
		get_from_seed::<AuraId>(s),
		get_from_seed::<GrandpaId>(s),
	)
}

pub fn authority_keys_from_ss58(s_aura :&str, s_grandpa : &str) -> (AuraId, GrandpaId) {
	(
		get_aura_from_ss58_addr(s_aura),
		get_grandpa_from_ss58_addr(s_grandpa),
	)
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || network_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![
				authority_keys_from_seed("Alice"),
			],
			// Sudo account
			AccountId::from_ss58check("5FhVHyw3jATLoUFYAgj1fSF79m7h4pdY6wku1yJ6Lvbx1uCJ").unwrap(), // Sudo
			// Pre-funded accounts
			vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
				get_account_id_from_seed::<sr25519::Public>("Dave"),
				get_account_id_from_seed::<sr25519::Public>("Eve"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			],
			true,
		),
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || network_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![
				authority_keys_from_seed("Alice"),
				authority_keys_from_seed("Bob"),
			],
			// Sudo account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// Pre-funded accounts
			vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
				get_account_id_from_seed::<sr25519::Public>("Dave"),
				get_account_id_from_seed::<sr25519::Public>("Eve"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
				get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
				get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
			],
			true,
		),
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		// Extensions
		None,
	))

}

/// *************************************
/// KUSANAGI MAIN NET CONFIGURATION
/// *************************************
pub fn kusanagi_mainnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		"Kusanagi bittensor main net",
		"kusanagi_mainnet",
		ChainType::Live,
		move || network_genesis(
			wasm_binary,
			vec![
				authority_keys_from_ss58("5CJEfuCe7QEztdnHJEBiXsfGynEAmnTfa5DDdiXxP7zPSqQG","5F7ia4UqyimKaJuvTQnFsRNJhSX2UiM5sFXcWmGjAPtDnwpc"), // jarvis
				authority_keys_from_ss58("5HgnTqtdTaAjjVPg2VRaHiytDJzG3Tr9bmEi46sk3pCxuKPo", "5FLdFNe3SnHCZxgRGJKQoBroeXWKPC48AkgHwH7rNoTEX5L4"), // genisys
				authority_keys_from_ss58("5Fsrw5m78ckJ2v5KqPo7QE6Axrmt5TatB2E16eD2ruAh3CCn", "5DQVooh4NVUpQdFkrv58myP5F7TTjMkMAF1YKxKJFJQE7FQc"), // hal
				authority_keys_from_ss58("5DRU4az6QWpBmNkAtsYMjaV5qRx1X9jLcZYKq7EKgM4noWn8", "5Do8YhJpniwzzDZchtNNynn5gzp32Syv7DHjVekpDdwtcUik"), // wopr
				authority_keys_from_ss58("5HdtW1gDZorC89Un8PzPpMXtYPTp4Fybst6mzLg1TDpL8hBm", "5H3jUpgQ5nJXszJxStR9LgRMf1qCyfn417pxh5pYzwwRukAo"), // gibson
				authority_keys_from_ss58("5Cz9opG7WReFPxu6EdvWaxDTQy5Lk8A4idLMYidijjhUq4sh", "5GgZ8yNBc5vWFJPUgEuDW3Q7vVEoBBwo2RFA8A2gJ6TKoeMs"), // glados
			],
			AccountId::from_ss58check("5GbSmaoza9rzDViaLTmFS2vhjobEQdv93cekXYAJ6XPstMej").unwrap(),
			vec![
				AccountId::from_ss58check("5FCJNwo2MSnHBEgoydnXt1aLGdFL6pCmpte476nFQ4X5vmxe").unwrap(), // Adam
			],
			true,
		),
		vec![
			MultiaddrWithPeerId::from_str("/dns4/anton.kusanagi.bittensor.com/tcp/30333/p2p/12D3KooWLUkry8sLj2zCVaNYbTsHW2EkKzh3AwKVocABAnaJGmSe").unwrap(),
			MultiaddrWithPeerId::from_str("/dns4/skynet.kusanagi.bittensor.com/tcp/30333/p2p/12D3KooWCo8hhxLKeitiWvsdYwVdw2UCRQi64JtZWFB8Bcp4E8RZ").unwrap()
	    ],
		None,
		None,
		None,
		None,
	))
}


/// *************************************
/// AKIRA TESTNET CONFIGURATION
/// *************************************
pub fn akira_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		"Akira bittensor testnet",
		"akira_testnet",
		ChainType::Local,
		move || network_genesis(
			wasm_binary,
			vec![
				authority_keys_from_ss58("5H9cxPkm15NEwUCS8rXKAQuq3z6hDDaBQfVneDme8tLP2NnR", "5EcstGNGzbZ8kLhpAdVSjT3So99hYCtV4ur8PPcdVaBQDpmR"), //Jarvis
				authority_keys_from_ss58("5H18kRHixaSSz9o1YeL4RBjm48YbcBW64wt9NQchbBzrMDFK", "5GuNfTJpx4NTyhwxZ2rLpndHnHau1qsoA16rruPppGTtKTRS"), //Genisys
				authority_keys_from_ss58("5CnyFHQhU2xeNWYCcBaEgsWyeyEj62xjotn1dHfaN2aWfdSK", "5DDsBzUkaNaLbYv5cewYuaaWWkvACoFguErRpQQqiEM4vndm"), //HAL
				authority_keys_from_ss58("5EnpbUbZ1kDmuefTm2t43K4TXBKu8cponanH8fXQCkPCPJ1j", "5ESo84zVXx7F6wuhfhD6qGVgF9ji2ShJFRR7GqewYnvC6bBB"), //WOPR
				authority_keys_from_ss58("5Dbz2iqzsP1dbHqsETLE7Kg8Xx4AakJobv71XU9gZh1Dvpb2", "5HLVPabG3pb33WjvwnEpbQivacF3WdodMN3GqJPbycGsPbY8"), //Gibson
				authority_keys_from_ss58("5F93P3nuzNdwGz8yTEoLTco3qknmzKkQYAkqbkJccunohevd", "5GaoAfPg71886Y6qZ2dEWFugmRGUhjLWiKz1tG7WE53e93Qz")  //Glados
			],
			AccountId::from_ss58check("5CRgsNaiCeGqSRZNGkVWu1rhs37cXQMyH4nbdfpHEwXivUQr").unwrap(), // Sudo
			vec![
				AccountId::from_ss58check("5DFtn3tjjTiQPopdt3behskR9U9Jc2MgvewCQdqbvdgoT9D5").unwrap(), // Adam
			],
			true,
		),
		vec![
			MultiaddrWithPeerId::from_str("/dns4/anton.akira.bittensor.com/tcp/30333/p2p/12D3KooWAcwbhijTx8NB5P9sLGcWyf4QrhScZrqkqWsh418Nuczd").unwrap(),
			MultiaddrWithPeerId::from_str("/dns4/skynet.akira.bittensor.com/tcp/30333/p2p/12D3KooWEr7Dq9oFJRSXZrZspibBLRySnGCDV7598xrGF8iT5DHD").unwrap()
	    ],
		None,
		None,
		None,
		None,
	))
}


/// Configure initial storage state for FRAME modules.
fn network_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		frame_system: Some(SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_balances: Some(BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k|(k, u128::pow(10,9))).collect(),
		}),
		pallet_aura: Some(AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		}),
		pallet_grandpa: Some(GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		}),
		pallet_sudo: Some(SudoConfig {
			// Assign network admin rights.
			key: root_key,
		}),
	}
}
