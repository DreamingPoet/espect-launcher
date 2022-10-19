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
    pub state: bool, // 运行状态
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
    
    let mut data = ClientData {
        id: -1, // 传递给服务器之后,由服务器设置
        name: "".to_string(),
        ip: "".to_string(),
        b_admin: false,
        client_op: ops,
        apps: get_local_apps(),
    };
    
    if let Some(ip) = local_ipaddress::get() {
        data.ip = ip;
    }

    let s = System::new_all();
    if let Some(device_name) = s.host_name(){
        data.name = device_name;
    }

    serde_json::to_string_pretty(&data).unwrap()
}

pub fn get_local_apps() -> Vec<ClientApp> {
    let path = env::current_dir().unwrap().join("apps");
    let mut apps = vec![];

    println!("current_dir = {:?}", path);
    let dir = path.as_path().read_dir().unwrap();
    for x in dir {
        if let Ok(path) = x {

            let mut folder = "".to_string();
            if path.path().is_dir() {
                folder = path.path().file_name().unwrap().to_str().unwrap().to_owned();

            }
            // println!("path {:?}", folder); // 该路径下所有文件和文件夹名称

            let sub_path = path.path().read_dir().unwrap();
            for j in sub_path {
                if let Ok(jj) = j {
                    if jj.path().is_file() && jj.path().to_str().unwrap().contains(".exe") {
                        println!("{:?}", jj.path().file_stem().unwrap().to_str()); // 该路径下所有文件和文件夹名称
                        
                        apps.push(ClientApp {
                            name: jj.path().file_stem().unwrap().to_str().unwrap().to_owned(),
                            folder: folder.clone(),
                            dscrpt: "".to_string(),
                            icon: "".to_string(),
                            state: false,
                        });
                    }
                }
            }
        }
    }
    apps
}


#[test]
fn test_get_local_apps() {
    let s = System::new_all();
    println!("sys info {:?}", s.networks());
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
    for _process in s.processes_by_exact_name(&(app_name.to_owned() + &".exe")) {
        // println!("{} {}", process.pid(), process.name());
        
        return true;
    }
    return false;
}

pub fn kill_app(app_name: &String) {
    println!("kill {}", app_name);
    let s = System::new_all();
    
    for process in s.processes_by_exact_name(&(app_name.to_owned() + &".exe")) {
        println!("kill {} {}", process.pid(), process.name());
        process.kill();
    }
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
    println!("local_apps_len = {:?}", local_apps.len());
    for i in local_apps {
        println!("local_apps = {:?}", i.name);
        let mut a = "false";
        if is_app_running(&i.name) {
            a = "true";
        }
        data.push(ClientStateData{ name: i.name.clone(), data:a.to_string() });
    }
    data
}
