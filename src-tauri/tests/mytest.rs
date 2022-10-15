
mod client_datas {
    use serde::{Serialize,Deserialize};

    #[derive(Serialize,Deserialize)]
    pub struct ClientFunc {
        func: String,
        data: ClientData,

    }

    #[derive(Serialize,Deserialize)]
    pub struct ClientData {
        name: String,
        ip: String,
        b_admin: bool,
        client_op: Vec<ClientOperation>,

    }

    #[derive(Serialize,Deserialize)]
    pub struct ClientOperation {
        name: String,
        dscrpt: String,
    }

    #[derive(Serialize,Deserialize)]
    pub struct ClientApp {
        pub name: String,
        pub dscrpt: String,
        pub icon: String,
    }
}

use crate::client_datas::ClientApp;

#[test]
fn mytest() {
    println!("this is my test");

    
    let data = ClientApp{
        name: "sadlkfj".to_string(),
        dscrpt: "sdfe".to_string(),
        icon: "data::64".to_string(),
    };

    let data_str = serde_json::to_string_pretty(&data).unwrap();
    println!("ClientApp = {}", data_str);

    let str_data:ClientApp =  serde_json::from_str(&data_str).unwrap();

    println!("{}", str_data.name);
    println!("{}", str_data.dscrpt);
    println!("{}", str_data.icon);
}


