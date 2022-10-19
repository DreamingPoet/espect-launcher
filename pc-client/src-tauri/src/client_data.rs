use std::{env, path::Path};

use serde::{Deserialize, Serialize};
use sysinfo::{ProcessExt, System, SystemExt};

#[derive(Serialize, Deserialize)]
pub struct ClientFunc {
    pub func_name: String,
    pub data: String,
}

#[derive(Serialize, Deserialize)]
pub struct ClientData {
    pub id: i32,
    pub name: String,
    pub ip: String,
    pub b_admin: bool,
    pub client_op: Vec<ClientOperation>,
    pub apps: Vec<ClientApp>,
}

#[derive(Serialize, Deserialize)]
pub struct ClientOperation {
    pub name: String,
    pub dscrpt: String,
}

#[derive(Serialize, Deserialize)]
pub struct ClientApp {
    pub name: String, // 名称
    pub folder: String, // 文件夹名称
    pub dscrpt: String, // 描述
    pub icon: String,
}

#[derive(Serialize, Deserialize)]
pub struct ClientStateData {
    pub name: String,
    pub data: String,
}

#[derive(Serialize, Deserialize)]
pub struct ClientUpdateData {
    states: Vec<ClientStateData>,
    apps: Vec<ClientStateData>,
}

pub fn get_local_data() -> String {
    let ops = vec![
        ClientOperation {
            name: "Start SteamVR".to_string(),
            dscrpt: "".to_string(),
        },
        ClientOperation {
            name: "Power Down".to_string(),
            dscrpt: "".to_string(),
        },
    ];

    let data = ClientData {
        id: -1, // 传递给服务器之后,由服务器设置
        name: "背包1".to_string(),
        ip: "127.0.0.1".to_string(),
        b_admin: false,
        client_op: ops,
        apps: get_local_apps(),
    };
    serde_json::to_string_pretty(&data).unwrap()
}

pub fn get_local_apps() -> Vec<ClientApp> {
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
                        apps.push(ClientApp {
                            name: jj.path().file_stem().unwrap().to_str().unwrap().to_owned(),
                            folder: jj.path().file_name().unwrap().to_str().unwrap().to_owned(),
                            dscrpt: "".to_string(),
                            icon: "".to_string(),
                        });
                    }
                }
            }
        }
    }
    apps
}

// 获取要更新的状态数据
pub(crate) fn get_update_data(local_apps: &Vec<ClientApp>) -> String {
    let data = ClientUpdateData{
        states: get_update_states(),
        apps: get_update_apps(local_apps),
    };
    serde_json::to_string_pretty(&data).unwrap()
}

pub fn is_app_running(app_name: &String) -> bool {
    let s = System::new_all();
    for process in s.processes_by_exact_name(&(app_name.to_owned() + &".exe")) {
        // println!("{} {}", process.pid(), process.name());
        return true;
    }
    return false;
}

// 获取要更新的状态数据
fn get_update_states() -> Vec<ClientStateData>{
    let state_data1 = ClientStateData{ name: "todo!()".to_string(), data: "todo!()".to_string() };
    let state_data2 = ClientStateData{ name: "todo!()".to_string(), data: "todo!()".to_string() };
    let state_data3 = ClientStateData{ name: "todo!()".to_string(), data: "todo!()".to_string() };
    let state_data4 = ClientStateData{ name: "todo!()".to_string(), data: "todo!()".to_string() };

    vec![state_data1, state_data2, state_data3, state_data4]

}

// 获取要更新的app 运行数据
fn get_update_apps(local_apps: &Vec<ClientApp>) -> Vec<ClientStateData> {
    let mut data:Vec<ClientStateData> = vec![];

    for i in local_apps {
        let mut a = "false";
        if is_app_running(&i.name) {
            a = "false";
        }
        data.push(ClientStateData{ name: i.dscrpt.clone(), data:a.to_string() });
    }
    data
}
