use std::convert::From;
use crate::aws::influx::InfluxInstance;
use crate::aws::rds::RdsInstance;
use crate::aws::vault::VaultInstance;

#[derive(Debug)]
pub enum AwsAccount {
    Dev,
    Stage,
    Prod,
}

impl From<&str> for AwsAccount {
    fn from(account_id: &str) -> Self {
        match account_id {
            "983257951706" => AwsAccount::Dev,
            "975050271628" => AwsAccount::Stage,
            "871891271706" => AwsAccount::Prod,
            _ => panic!("Unknown AWS sso account id: {account_id}"),
        }
    }
}

impl AwsAccount {
    pub fn vault_instance(&self) -> VaultInstance {
        match self {
            AwsAccount::Dev => VaultInstance::NonProd,
            AwsAccount::Stage => VaultInstance::NonProd,
            AwsAccount::Prod => VaultInstance::Prod,
        }
    }

    pub fn influx_instances(&self) -> Vec<InfluxInstance> {
        match self {
            AwsAccount::Dev => vec![InfluxInstance::MetricsDev],
            AwsAccount::Stage => vec![InfluxInstance::MetricsStage],
            AwsAccount::Prod => vec![InfluxInstance::MetricsProd],
        }
    }

    pub fn rds_instances(&self) -> Vec<RdsInstance> {
        match self {
            AwsAccount::Dev => vec![RdsInstance::WorkcellDev, RdsInstance::EventLogDev],
            AwsAccount::Stage => vec![RdsInstance::WorkcellStage, RdsInstance::EventLogStage],
            AwsAccount::Prod => vec![RdsInstance::WorkcellProd, RdsInstance::EventLogProd],
        }
    }
}
