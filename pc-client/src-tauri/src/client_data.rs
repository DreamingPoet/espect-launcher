use std::env;

use serde::{Serialize,Deserialize};

#[derive(Serialize,Deserialize)]
pub struct ClientFunc {
    pub func_name: String,
    pub data: String,
}

#[derive(Serialize,Deserialize)]
pub struct ClientData {
    pub id: i32,
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
    
    let path = env::current_dir().unwrap().join("apps");

    let mut apps = vec![];

    println!("current_dir = {:?}", path);
    let dir = path.as_path().read_dir().unwrap();
    for x in dir {
    if let Ok(path) = x {
        // println!("{:?}", path.path()); // 该路径下所有文件和文件夹名称
        let sub_path = path.path().read_dir().unwrap();
        for j in sub_path {
            if let Ok(jj) = j {
                if jj.path().is_file() && jj.path().to_str().unwrap().contains(".exe") {
                    println!("{:?}", jj.path().file_stem().unwrap().to_str()); // 该路径下所有文件和文件夹名称
                    let name = jj.path().to_str().unwrap().to_owned();
                    apps.push(ClientApp { name: name, dscrpt: jj.path().file_stem().unwrap().to_str().unwrap().to_owned(), icon: "".to_string() });
                }

            }
        }
        }
    }


    let ops = vec![
        ClientOperation { name: "Start SteamVR".to_string(), dscrpt: "".to_string() },
        ClientOperation { name: "Power Down".to_string(), dscrpt: "".to_string() },
    ];


    let data = ClientData {
        id:-1, // 传递给服务器之后,由服务器设置
        name: "背包1".to_string(),
        ip: "127.0.0.1".to_string(),
        b_admin: false,
        client_op: ops ,
        apps:apps,
    };
    serde_json::to_string_pretty(&data).unwrap()
}


