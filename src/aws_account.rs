use std::convert::From;

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
