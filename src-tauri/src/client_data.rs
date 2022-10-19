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
    pub name: String, // 名称
    pub folder: String, // 文件夹名称
    pub dscrpt: String, // 描述
    pub icon: String,
}