use cosmos_sdk_proto::cosmos::auth::v1beta1::query_client::QueryClient;
use cosmos_sdk_proto::cosmos::auth::v1beta1::{
    BaseAccount, QueryAccountRequest, QueryAccountResponse,
};
use cosmos_sdk_proto::cosmos::vesting::v1beta1::PeriodicVestingAccount;

use tonic::transport::Channel;

pub async fn query_account(channel: Channel, address: String) -> anyhow::Result<BaseAccount> {
    let res: QueryAccountResponse = QueryClient::new(channel)
        .account(QueryAccountRequest { address: address })
        .await?
        .into_inner();
    //println!("{:?}", res.account.as_ref().unwrap().value);
    //println!("{:?}", res.account.as_ref().unwrap().type_url);
    if let Some(account) = &res.account.as_ref() {
        if account.type_url == "/cosmos.vesting.v1beta1.PeriodicVestingAccount" {
            let periodic_vesting_account: PeriodicVestingAccount =
                cosmos_sdk_proto::traits::MessageExt::from_any(&res.account.as_ref().unwrap())
                    .unwrap();
            //println!("{:?}", periodic_vesting_account);

            let base_vesting_account = periodic_vesting_account.base_vesting_account.unwrap();
            let base_account = base_vesting_account.base_account.unwrap();
            return Ok(base_account);
        } else if account.type_url == "/cosmos.auth.v1beta1.BaseAccount" {
            let base_account: BaseAccount =
                cosmos_sdk_proto::traits::MessageExt::from_any(&res.account.as_ref().unwrap())
                    .unwrap();
            return Ok(base_account);
        } else if account.type_url == "/cosmos.auth.v1beta1.ModuleAccount" {
            return Err(anyhow::anyhow!("Error: No handler for this account type."));
        }
    }
    return Err(anyhow::anyhow!("Error"));
}
