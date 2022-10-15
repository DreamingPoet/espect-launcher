use serde::{Serialize,Deserialize};

#[derive(Serialize,Deserialize)]
pub struct ClientFunc {
    pub func_name: String,
    pub data: String,
}

#[derive(Serialize,Deserialize)]
pub struct ClientData {
    pub name: String,
    pub ip: String,
    pub b_admin: bool,
    pub client_op: Vec<ClientOperation>,
    pub apps: Vec<ClientApp>,

}

#[derive(Serialize,Deserialize)]
pub struct ClientOperation {
    pub name: String,
    pub dscrpt: String,
}

#[derive(Serialize,Deserialize)]
pub struct ClientApp {
    pub name: String,
    pub dscrpt: String,
    pub icon: String,
}


pub fn get_local_data() -> String {
    let ops = vec![
        ClientOperation { name: "Start SteamVR".to_string(), dscrpt: "".to_string() },
        ClientOperation { name: "Power Down".to_string(), dscrpt: "".to_string() },
    ];

    let apps = vec![
        ClientApp { name: "导游培训".to_string(), dscrpt: "".to_string(), icon: "".to_string() },
        ClientApp { name: "试听实验".to_string(), dscrpt: "".to_string(), icon: "".to_string() },
    ];

    let data = ClientData {
        name: "背包1".to_string(),
        ip: "127.0.0.1".to_string(),
        b_admin: false,
        client_op: ops ,
        apps:apps,
    };
    serde_json::to_string_pretty(&data).unwrap()
}