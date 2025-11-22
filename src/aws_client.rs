// src/aws_client.rs

use anyhow::Result;
use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_config::Region;
use aws_sdk_cloudwatch::Client;

/// Build a CloudWatch client, optionally overriding the region.
///
/// If `region` is `None`, this respects:
/// - AWS_REGION / AWS_DEFAULT_REGION
/// - profile / config files
/// - IMDS, etc.
///
/// If `region` is `Some("eu-central-1")`, that wins.
pub async fn make_client(region: Option<&str>) -> Result<Client> {
    let region_provider = match region {
        Some(explicit) => {
            // Prefer explicit region, but still fall back to default provider if something’s off
            RegionProviderChain::first_try(Region::new(explicit.to_string()))
                .or_default_provider()
        }
        None => RegionProviderChain::default_provider(),
    };

    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;

    Ok(Client::new(&config))
}
