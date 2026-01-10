use std::convert::From;
use crate::aws::influx::InfluxInstance;
use crate::aws::rds::RdsInstance;
use crate::aws::vault::VaultInstance;

#[derive(Debug)]
pub enum AwsAccount {
    DataPlatform,
    Dev,
    Iot,
    Prod,
    Sandbox,
    Stage,
}

impl From<&str> for AwsAccount {
    fn from(account_id: &str) -> Self {
        match account_id {
            "789472542317" => AwsAccount::DataPlatform,
            "983257951706" => AwsAccount::Dev,
            "283152483325" => AwsAccount::Iot,
            "871891271706" => AwsAccount::Prod,
            "287642671827" => AwsAccount::Sandbox,
            "975050271628" => AwsAccount::Stage,
            _ => panic!("Unknown AWS sso account id: {account_id}"),
        }
    }
}

impl AwsAccount {
    pub fn vault_instance(&self) -> VaultInstance {
        match self {
            AwsAccount::DataPlatform => VaultInstance::Prod,
            AwsAccount::Dev => VaultInstance::NonProd,
            AwsAccount::Prod => VaultInstance::Prod,
            AwsAccount::Sandbox => VaultInstance::NonProd,
            AwsAccount::Stage => VaultInstance::NonProd,
            _ => panic!("No Vault instance exists for this AWS account: {:?}", self),
        }
    }

    pub fn influx_instances(&self) -> Vec<InfluxInstance> {
        match self {
            AwsAccount::Dev => vec![InfluxInstance::MetricsDev],
            AwsAccount::Prod => vec![InfluxInstance::MetricsProd],
            AwsAccount::Stage => vec![InfluxInstance::MetricsStage],
            _ => panic!("No Influx instances exist for this AWS account: {:?}", self),
        }
    }

    pub fn rds_instances(&self) -> Vec<RdsInstance> {
        match self {
            AwsAccount::Dev => vec![RdsInstance::WorkcellDev, RdsInstance::EventLogDev],
            AwsAccount::Prod => vec![RdsInstance::WorkcellProd, RdsInstance::EventLogProd],
            AwsAccount::Stage => vec![RdsInstance::WorkcellStage, RdsInstance::EventLogStage],
            _ => panic!("No RDS instances exist for this AWS account: {:?}", self),
        }
    }
}
