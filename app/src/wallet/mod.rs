use uuid::Uuid;

pub struct Wallet {
    pub wallet_id: Uuid,
    pub name: String,
    pub chains: Vec<String>,
    pub wallet_type: WalletType,
}

pub enum WalletType {
    Safe {
        address: String,
    },
    EOA {
        address: String,
    },
    View {
        address: String,
    },
    Railgun {
        railgun_address: String,
    }
}
