use crate::utils::{
    nft_abi_calls::{approve, mint},
    test_helpers::{defaults, setup},
    token_distributor_abi_calls::{create, set_reserve, token_distribution},
};
use fuels::{prelude::Identity, signers::Signer};

mod success {

    use super::*;

    #[tokio::test]
    async fn sets_reserve() {
        let (
            _deployer,
            owner1,
            _owner2,
            _token_distributor_contract,
            fractional_nft_contract,
            nft_contract,
            asset_contract,
        ) = setup().await;
        let (reserve_price, token_price, token_supply, _purchase_amount, _asset_supply) =
            defaults().await;

        let owner_identity = Identity::Address(owner1.wallet.address().into());
        let fractional_nft_identity = Identity::ContractId(fractional_nft_contract.into());

        mint(1, &owner1.nft, owner_identity.clone()).await;
        approve(Some(fractional_nft_identity.clone()), &owner1.nft, 0).await;
        create(
            &owner1.token_distributor,
            asset_contract.clone(),
            fractional_nft_contract.clone(),
            nft_contract.clone(),
            Some(reserve_price),
            Some(owner_identity.clone()),
            token_price,
            token_supply,
            0,
        )
        .await;

        let token_distribution_struct =
            token_distribution(&owner1.token_distributor, fractional_nft_contract.clone()).await;
        assert_eq!(
            token_distribution_struct.clone().unwrap().reserve_price,
            Some(reserve_price)
        );

        set_reserve(
            &owner1.token_distributor,
            fractional_nft_contract.clone(),
            Some(reserve_price + 1),
        )
        .await;

        let token_distribution_struct =
            token_distribution(&owner1.token_distributor, fractional_nft_contract.clone()).await;
        assert_eq!(
            token_distribution_struct.clone().unwrap().reserve_price,
            Some(reserve_price + 1)
        );
    }
}

mod revert {

    use super::*;

    #[tokio::test]
    #[should_panic(expected = "DistributionDoesNotExist")]
    async fn when_token_distribution_does_not_exist() {
        let (
            _deployer,
            owner1,
            _owner2,
            _token_distributor_contract,
            fractional_nft_contract,
            _nft_contract,
            _asset_contract,
        ) = setup().await;
        let (reserve_price, _token_price, _token_supply, _purchase_amount, _asset_supply) =
            defaults().await;

        set_reserve(
            &owner1.token_distributor,
            fractional_nft_contract.clone(),
            Some(reserve_price + 1),
        )
        .await;
    }

    #[tokio::test]
    #[should_panic(expected = "NotTokenAdmin")]
    async fn when_not_owner() {
        let (
            _deployer,
            owner1,
            _owner2,
            _token_distributor_contract,
            fractional_nft_contract,
            nft_contract,
            asset_contract,
        ) = setup().await;
        let (reserve_price, token_price, token_supply, _purchase_amount, _asset_supply) =
            defaults().await;

        let owner_identity = Identity::Address(owner1.wallet.address().into());
        let fractional_nft_identity = Identity::ContractId(fractional_nft_contract.into());

        mint(1, &owner1.nft, owner_identity.clone()).await;
        approve(Some(fractional_nft_identity.clone()), &owner1.nft, 0).await;
        create(
            &owner1.token_distributor,
            asset_contract.clone(),
            fractional_nft_contract.clone(),
            nft_contract.clone(),
            Some(reserve_price),
            None,
            token_price,
            token_supply,
            0,
        )
        .await;

        set_reserve(
            &owner1.token_distributor,
            fractional_nft_contract.clone(),
            Some(reserve_price + 1),
        )
        .await;
    }
}