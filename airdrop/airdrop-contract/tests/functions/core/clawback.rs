use crate::utils::{
    interface::core::{airdrop_constructor, claim, clawback},
    setup::{build_tree, defaults, get_wallet_balance, setup},
};

mod success {

    use super::*;

    #[tokio::test]
    async fn returns_all_coins() {
        let (deploy_wallet, wallet1, wallet2, wallet3, asset_id) = setup().await;
        let (
            _,
            _,
            _,
            minter,
            key,
            num_leaves,
            asset_supply,
            airdrop_leaves,
            claim_time,
            _,
            original_balance,
        ) = defaults(&deploy_wallet, &wallet1, &wallet2, &wallet3).await;

        let (_tree, root, _leaf, _) = build_tree(key, airdrop_leaves.to_vec()).await;
        let provider = deploy_wallet.wallet.provider().unwrap();

        airdrop_constructor(
            minter,
            asset_supply,
            asset_id,
            claim_time,
            &deploy_wallet.airdrop_distributor,
            root,
            num_leaves,
        )
        .await;

        let _ = provider.produce_blocks(claim_time + 1, Option::None).await;

        assert_eq!(
            get_wallet_balance(&deploy_wallet.wallet, &asset_id).await,
            original_balance - asset_supply
        );

        clawback(&deploy_wallet.airdrop_distributor).await;

        assert_eq!(
            get_wallet_balance(&deploy_wallet.wallet, &asset_id).await,
            original_balance
        );
    }

    #[tokio::test]
    async fn returns_unclaimed_coins() {
        let (deploy_wallet, wallet1, wallet2, wallet3, asset_id) = setup().await;
        let (
            identity_a,
            _,
            _,
            minter,
            key,
            num_leaves,
            asset_supply,
            airdrop_leaves,
            claim_time,
            _,
            original_balance,
        ) = defaults(&deploy_wallet, &wallet1, &wallet2, &wallet3).await;

        let (_tree, root, _leaf, proof) = build_tree(key, airdrop_leaves.to_vec()).await;
        let provider = deploy_wallet.wallet.provider().unwrap();

        airdrop_constructor(
            minter,
            asset_supply,
            asset_id,
            claim_time,
            &deploy_wallet.airdrop_distributor,
            root,
            num_leaves,
        )
        .await;

        claim(
            airdrop_leaves[key as usize].1,
            &wallet1.airdrop_distributor,
            key,
            proof.clone(),
            identity_a,
        )
        .await;

        let _ = provider.produce_blocks(claim_time + 1, Option::None).await;

        assert_eq!(
            get_wallet_balance(&deploy_wallet.wallet, &asset_id).await,
            original_balance - asset_supply
        );

        clawback(&deploy_wallet.airdrop_distributor).await;

        assert_eq!(
            get_wallet_balance(&deploy_wallet.wallet, &asset_id).await,
            original_balance - airdrop_leaves[key as usize].1
        );
    }
}

mod revert {

    use super::*;

    #[tokio::test]
    #[should_panic(expected = "CallerNotAdmin")]
    async fn when_not_admin() {
        let (deploy_wallet, wallet1, wallet2, wallet3, asset_id) = setup().await;
        let (_, _, _, minter, key, num_leaves, asset_supply, airdrop_leaves, claim_time, _, _) =
            defaults(&deploy_wallet, &wallet1, &wallet2, &wallet3).await;

        let (_tree, root, _leaf, _) = build_tree(key, airdrop_leaves.to_vec()).await;
        let provider = deploy_wallet.wallet.provider().unwrap();

        airdrop_constructor(
            minter,
            asset_supply,
            asset_id,
            claim_time,
            &deploy_wallet.airdrop_distributor,
            root,
            num_leaves,
        )
        .await;

        let _ = provider.produce_blocks(claim_time + 1, Option::None).await;

        clawback(&wallet1.airdrop_distributor).await;
    }

    #[tokio::test]
    #[should_panic(expected = "ClaimPeriodActive")]
    async fn when_in_claim_period() {
        let (deploy_wallet, wallet1, wallet2, wallet3, asset_id) = setup().await;
        let (_, _, _, minter, key, num_leaves, asset_supply, airdrop_leaves, claim_time, _, _) =
            defaults(&deploy_wallet, &wallet1, &wallet2, &wallet3).await;

        let (_tree, root, _leaf, _) = build_tree(key, airdrop_leaves.to_vec()).await;

        airdrop_constructor(
            minter,
            asset_supply,
            asset_id,
            claim_time,
            &deploy_wallet.airdrop_distributor,
            root,
            num_leaves,
        )
        .await;

        clawback(&deploy_wallet.airdrop_distributor).await;
    }

    #[tokio::test]
    #[should_panic(expected = "CallerNotAdmin")]
    async fn when_not_initalized() {
        let (deploy_wallet, wallet1, wallet2, wallet3, _) = setup().await;
        let (_, _, _, _, key, _, _, airdrop_leaves, _, _, _) =
            defaults(&deploy_wallet, &wallet1, &wallet2, &wallet3).await;

        let (_tree, _, _leaf, _) = build_tree(key, airdrop_leaves.to_vec()).await;

        clawback(&deploy_wallet.airdrop_distributor).await;
    }

    #[tokio::test]
    #[should_panic(expected = "NotEnoughCoins")]
    async fn when_called_twice() {
        let (deploy_wallet, wallet1, wallet2, wallet3, asset_id) = setup().await;
        let (_, _, _, minter, key, num_leaves, asset_supply, airdrop_leaves, claim_time, _, _) =
            defaults(&deploy_wallet, &wallet1, &wallet2, &wallet3).await;

        let (_tree, root, _leaf, _) = build_tree(key, airdrop_leaves.to_vec()).await;
        let provider = deploy_wallet.wallet.provider().unwrap();

        airdrop_constructor(
            minter,
            asset_supply,
            asset_id,
            claim_time,
            &deploy_wallet.airdrop_distributor,
            root,
            num_leaves,
        )
        .await;

        let _ = provider.produce_blocks(claim_time + 1, Option::None).await;

        clawback(&deploy_wallet.airdrop_distributor).await;
        clawback(&deploy_wallet.airdrop_distributor).await;
    }
}
