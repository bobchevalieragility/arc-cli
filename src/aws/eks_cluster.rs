use std::convert::From;

pub enum EksCluster {
    Dev,
    Prod,
    Sandbox,
    Stage,
}

impl From<&str> for EksCluster {
    fn from(cluster_name: &str) -> Self {
        match cluster_name {
            "tailscale-operator-platform-dev-uw2.tail5a6c.ts.net" => EksCluster::Dev,
            // "arn:aws:eks:us-west-2:983257951706:cluster/platform-dev-uw2" => EksCluster::Dev,
            "tailscale-operator-platform-prod-uw2.tail5a6c.ts.net" => EksCluster::Prod,
            "tailscale-operator-platform-stage-uw2.tail5a6c.ts.net" => EksCluster::Stage,
            "tailscale-operator-sandbox-uw2.tail5a6c.ts.net" => EksCluster::Sandbox,
            _ => panic!("Unknown EKS cluster name: {cluster_name}"),
        }
    }
}

impl EksCluster {
    pub fn namespace(&self) -> String {
        match self {
            EksCluster::Dev => "development".to_string(),
            EksCluster::Prod => "production".to_string(),
            EksCluster::Stage => "staging".to_string(),
            EksCluster::Sandbox => "sandbox".to_string(),
        }
    }
}
