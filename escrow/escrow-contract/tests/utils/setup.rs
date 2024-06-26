use fuels::{
    accounts::ViewOnlyAccount,
    prelude::{
        abigen, launch_custom_provider_and_get_wallets, Address, AssetConfig, AssetId, Contract,
        LoadConfiguration, StorageConfiguration, TxPolicies, WalletUnlocked, WalletsConfig,
    },
    test_helpers::ChainConfig,
    tx::{ConsensusParameters, ContractParameters, TxParameters},
    types::Identity,
};

abigen!(Contract(
    name = "Escrow",
    abi = "./escrow-contract/out/debug/escrow-contract-abi.json"
),);

const ESCROW_CONTRACT_BINARY_PATH: &str = "./out/debug/escrow-contract.bin";
const ESCROW_CONTRACT_STORAGE_PATH: &str = "./out/debug/escrow-contract-storage_slots.json";

pub(crate) struct Defaults {
    pub(crate) asset_amount: u64,
    pub(crate) asset_id: AssetId,
    pub(crate) deadline: u64,
    pub(crate) initial_wallet_amount: u64,
    pub(crate) other_asset_id: AssetId,
}

pub(crate) struct User {
    pub(crate) contract: Escrow<WalletUnlocked>,
    pub(crate) wallet: WalletUnlocked,
}

pub(crate) async fn asset_amount(asset: &AssetId, user: &User) -> u64 {
    user.wallet.clone().get_asset_balance(asset).await.unwrap()
}

pub(crate) async fn create_arbiter(user: &User, asset: AssetId, fee_amount: u64) -> Arbiter {
    Arbiter {
        address: Identity::Address(user.wallet.address().into()),
        asset,
        fee_amount,
    }
}

pub(crate) async fn create_asset(amount: u64, id: AssetId) -> Asset {
    Asset { amount, id }
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn escrow_info(
    arbiter: Arbiter,
    asset_count: u64,
    buyer: &User,
    asset: Option<AssetId>,
    deposited_amount: u64,
    deadline: u64,
    disputed: bool,
    first_asset_index: u64,
    seller: &User,
    state: bool,
) -> EscrowInfo {
    EscrowInfo {
        arbiter,
        asset_count,
        buyer: Buyer {
            address: Identity::Address(Address::from(buyer.wallet.address())),
            asset,
            deposited_amount,
        },
        deadline,
        disputed,
        first_asset_index,
        seller: Seller {
            address: Identity::Address(Address::from(seller.wallet.address())),
        },
        state: match state {
            true => State::Completed,
            false => State::Pending,
        },
    }
}

pub(crate) async fn setup() -> (User, User, User, Defaults) {
    let number_of_coins = 1;
    let coin_amount = 1_000_000;
    let number_of_wallets = 4;

    let base_asset = AssetConfig {
        id: AssetId::zeroed(),
        num_coins: number_of_coins,
        coin_amount,
    };
    let asset_id = AssetId::new([1; 32]);
    let asset = AssetConfig {
        id: asset_id,
        num_coins: number_of_coins,
        coin_amount,
    };
    let other_asset_id = AssetId::new([2; 32]);
    let other_asset = AssetConfig {
        id: other_asset_id,
        num_coins: number_of_coins,
        coin_amount,
    };
    let assets = vec![base_asset, asset, other_asset];

    let wallet_config = WalletsConfig::new_multiple_assets(number_of_wallets, assets);

    let mut consensus_parameters = ConsensusParameters::default();

    let tx_params = TxParameters::default().with_max_size(100_000_000);
    consensus_parameters.set_tx_params(tx_params);

    let contract_params = ContractParameters::default().with_contract_max_size(10_000_000);
    consensus_parameters.set_contract_params(contract_params);

    let chain_config = ChainConfig {
        consensus_parameters,
        ..ChainConfig::local_testnet()
    };

    let mut wallets =
        launch_custom_provider_and_get_wallets(wallet_config, None, Some(chain_config))
            .await
            .unwrap();

    let deployer_wallet = wallets.pop().unwrap();
    let arbiter_wallet = wallets.pop().unwrap();
    let buyer_wallet = wallets.pop().unwrap();
    let seller_wallet = wallets.pop().unwrap();

    let escrow_storage_configuration =
        StorageConfiguration::default().add_slot_overrides_from_file(ESCROW_CONTRACT_STORAGE_PATH);
    let escrow_configuration = LoadConfiguration::default()
        .with_storage_configuration(escrow_storage_configuration.unwrap());
    let escrow_id = Contract::load_from(ESCROW_CONTRACT_BINARY_PATH, escrow_configuration)
        .unwrap()
        .deploy(&deployer_wallet, TxPolicies::default())
        .await
        .unwrap();

    let arbiter = User {
        contract: Escrow::new(escrow_id.clone(), arbiter_wallet.clone()),
        wallet: arbiter_wallet,
    };
    let buyer = User {
        contract: Escrow::new(escrow_id.clone(), buyer_wallet.clone()),
        wallet: buyer_wallet,
    };
    let seller = User {
        contract: Escrow::new(escrow_id, seller_wallet.clone()),
        wallet: seller_wallet,
    };

    let defaults = Defaults {
        asset_id,
        asset_amount: 100,
        deadline: 100,
        initial_wallet_amount: coin_amount,
        other_asset_id,
    };

    (arbiter, buyer, seller, defaults)
}
