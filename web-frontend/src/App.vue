<script setup lang="ts">
import { onMounted, onUnmounted, reactive } from 'vue'
import {Setting, Opportunity } from '@element-plus/icons-vue'

let socket: any;
let get_all_clients_timer: any;
let update_client_timer: any;
let all_clients_data_hash = "null";
//  页面中绑定的数据
let client_states: ClientStateData[] = [];
const data = reactive({
  clients: new Map(),
  current_client_id: -1,
  menu_activeIndex: 1,
  client_states: client_states,

});


onMounted(() => {
  // http://localhost:3001/?server=192.168.0.1:3000
  let server = "ws://"+ window.location.host +"/ws";
  console.log("WebSocket server = " + server);
  socket = new WebSocket(server);
  socket.onopen = socket_onopen;
  socket.onmessage = socket_onmessage;
  socket.onclose = socket_onclose;
  socket.onerror = socket_onerror;


});

onUnmounted(() => {
  clear();
});

// 清理
const clear = () => {
  if (get_all_clients_timer) {
    clearInterval(get_all_clients_timer);
  }

  if (update_client_timer) {
    clearInterval(update_client_timer);
  }

};

function getQueryVariable(variable: string) {
  var query = window.location.search.substring(1);
  var vars = query.split("&");
  for (var i = 0; i < vars.length; i++) {
    var pair = vars[i].split("=");
    if (pair[0] == variable) { return pair[1]; }
  }
  return (false);
}


const socket_onopen = function (event: any) {
  console.log("[open] Connection established");
  socket.send("start_connection");

  get_all_clients(all_clients_data_hash);
  get_all_clients_timer = setInterval(() => {
    get_all_clients(all_clients_data_hash);
  }, 3000);

  update_client();
  update_client_timer = setInterval(() => {
    update_client();
  }, 2000);


};

// 收到消息, 只接收 ClientFunc 类型的数据
const socket_onmessage = function (event: any) {
  console.log(`[message] Data received from server: ${event.data}`);
  let func_data: ClientFunc = JSON.parse(event.data); // eval("(" + event.data + ")");
  if (func_data.func_name == "on_get_all_clients") {
    data.clients.clear();
    let all_clients_data:AllClientsData = JSON.parse(func_data.data.toString()); // AllClientsData
    // 保存上一次的hash
    all_clients_data_hash = all_clients_data.data_hash;
    for (let i of all_clients_data.all_clients) {
      let j:ClientData = JSON.parse(i);
      data.clients.set(j.id, j);
    }

  } else if (func_data.func_name == "on_update_client") {

    let update_data: ClientUpdateData = JSON.parse(func_data.data.toString());
    data.client_states = update_data.states;
    // console.log(data.clients.get(data.current_client_id).apps)
    for (let i of update_data.apps) {
      if (i.data == "true") {
        for (let j of data.clients.get(data.current_client_id).apps) {
          if (j.name == i.name) {
            console.log(j.name + "set true")
            j.state = true;
          }
        }
      } else {
        for (let j of data.clients.get(data.current_client_id).apps) {
          if (j.name == i.name) {
            console.log(j.name + "set false")
            j.state = false;
          }
        }
      }
    }
  }
};

const socket_onclose = function (event: any) {
  if (event.wasClean) {
    console.log(`[close] Connection closed, code=${event.code} reason=${event.reason}`);
  } else {
    console.log('[close] Connection died');
  }

  clear();
};

const socket_onerror = function (error: any) {
  console.log(error)
};

const sayhello = function () {
  socket.send('hello');
};

const get_all_clients = (data_hash :string) => {

  let data: ClientFunc = {
    func_name: "get_all_clients",
    data: data_hash,
  };
  let data_str = JSON.stringify(data);
  console.log(data_str);
  socket.send(data_str);

};

// 更新客户端信息
const update_client = () => {
  if (data.current_client_id < 0) return;

  let update_client: UpdateClient = {
    id: data.clients.get(data.current_client_id).id,
  }

  let func: ClientFunc = {
    func_name: "update_client",
    data: JSON.stringify(update_client),
  };
  let func_str = JSON.stringify(func);
  socket.send(func_str);
};

const call_option = function (index: number) {
  console.log("call index = " + index)
  data.current_client_id = index;
};


// 启动app
const start_app = (app_folder:string, app_name: string, start: boolean) => {
  let app_path = app_folder + app_name;
  let start_app_str: StartApp = {
    id: data.clients.get(data.current_client_id).id,
    start: start,
    app: app_path
  }
  let func: ClientFunc = {
    func_name: "start_app",
    data: JSON.stringify(start_app_str),
  };
  let func_str = JSON.stringify(func);
  socket.send(func_str);


  for (let i of data.clients.get(data.current_client_id).apps) {
      if (i.name == app_name) {
        i.running = !start;
      }
  }
};

// ======== data start ========

class ClientOperation {
  name: string;
  dscrpt: string;
  // 构造函数
  constructor(name: string, dscrpt: string) {
    this.name = name
    this.dscrpt = dscrpt
  }
}

class ClientApp {
  name: string;
  folder: string; // 文件夹名称
  dscrpt: string;
  icon: string;
  state: boolean;
  // 构造函数
  constructor(name: string, folder: string, dscrpt: string, icon: string, state: boolean) {
    this.name = name;
    this.folder = folder;
    this.dscrpt = dscrpt;
    this.icon = icon;
    this.state = state;

  }
}


class ClientData {
  id: number;
  name: string;
  ip: string;
  b_admin: boolean;
  client_op: ClientOperation[];
  apps: ClientApp[];

  // 构造函数
  constructor(
    id: number,
    name: string,
    ip: string,
    b_admin: boolean,
    client_op: ClientOperation[],
    apps: ClientApp[]
  ) {
    this.id = id;
    this.name = name;
    this.ip = ip;
    this.b_admin = b_admin;
    this.client_op = client_op;
    this.apps = apps;
  }
}

class ClientStateData {
  name: string;
  data: string;

  // 构造函数
  constructor(
    name: string,
    data: string
  ) {
    this.name = name;
    this.data = data;

  }
}

class ClientUpdateData {
  states: ClientStateData[];
  apps: ClientStateData[];

  // 构造函数
  constructor(
    states: ClientStateData[],
    apps: ClientStateData[]
  ) {
    this.states = states;
    this.apps = apps;
  }
}

class ClientFunc {
  func_name: String;
  data: String;
  // 构造函数
  constructor(func_name: string, data: string) {
    this.func_name = func_name
    this.data = data
  }
}

class StartApp {
  id: Number;
  start: boolean;
  app: string;
  // 构造函数
  constructor(id: Number, start: boolean, app: string) {
    this.id = id;
    this.start = start;
    this.app = app;
  }
}

class UpdateClient {
  id: Number;
  // 构造函数
  constructor(id: Number) {
    this.id = id;
  }
}


class AllClientsData {
  data_hash:string;
  all_clients:string[]; //json string of ClientData[]
  // 构造函数
  constructor(data_hash: string, all_clients:string[]) {
    this.data_hash = data_hash;
    this.all_clients = all_clients;
  }

}



// ======== data end ========

</script>

<template>
  <el-container class="main-container" style="height: 580px">
    <el-header class="main-header">


      <div class="title">
        <p class="app-name"> Espect 应用管理系统</p>
        <p class="app-version">v1.0.0</p>
      </div>

      <el-icon class="app-setting" @click="">
        <Setting />
      </el-icon>

    </el-header>

    <el-container>
      <el-aside>
        <el-scrollbar>
          <el-menu active-text-color="#ff8000" default-active="1">
            <el-menu-item v-for="[key, value] in data.clients" :index="key" @click="call_option(key)">
              设备 {{ value.id }}
            </el-menu-item>
          </el-menu>
        </el-scrollbar>
      </el-aside>

      <el-main class="client-content">

        <!-- 内部容器 main-->
        <el-scrollbar>
          <div v-if="data.clients.get(data.current_client_id)">
            <div class="client-info">

              <p>计算机名称: {{data.clients.get(data.current_client_id).name}}</p>
              <p>IP地址: {{data.clients.get(data.current_client_id).ip}}</p>

            </div>

            <!-- <div class="client-state">

              <div class="client-state-item" v-for="item in data.client_states"> 状态 <el-icon>
                  <Opportunity />
                </el-icon>
              </div>

            </div>

            <div class="client-option">

              <el-button v-for="item in data.clients.get(data.current_client_id).client_op" type="primary" plain>
                {{item.name}}</el-button>

            </div> -->

            <div class="client-apps">

              <el-card class="client-apps-item" v-for="(item, index) in data.clients.get(data.current_client_id).apps"
                body-style="padding:15px">
                <img src="./assets/exe.png" class="image" />
                <div class="apps-item-body">
                  <div class="apps-item-name">{{item.name}}</div>
                  <el-button v-if="item.state" type="danger" @click="start_app('', item.name, false)">关闭</el-button>

                  <el-button v-if="!item.state" type="primary" @click="start_app(item.folder + '/', item.name, true)">
                    启动</el-button>

                </div>
              </el-card>

            </div>

          </div>

        </el-scrollbar>

      </el-main>

    </el-container>
  </el-container>
</template>



<style lang="scss" scoped>
.main-container {
  .el-aside {
    width: 100px;
    background-color: #e7e7e7;
  }
}

.main-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background-color: #005989;
  color: #fff;

  .title {
    display: flex;
    align-items: baseline;

    .app-name {
      font-size: 20px;
    }

    .app-version {
      padding-left: 10px;
      font-size: 10px;
    }
  }

  .app-setting {
    font-size: 25px;
  }
}

.el-aside {
  .el-menu-item {
    background-color: #e7e7e7;

    &:hover {
      background-color: #ffffff;
    }

  }
}

.client-content {

  .client-info {
    font-size: 12px;
  }

  .client-state {
    display: flex;
    margin-top: 20px;

    .client-state-item {
      width: 160px;
    }

    .el-icon {
      color: #00ff00;
    }
  }

  .client-option {
    display: flex;
    flex-wrap: wrap;
    margin-top: 20px;
  }

  .client-apps {
    display: flex;
    flex-wrap: wrap;


    .client-apps-item {
      width: 160px;
      margin-top: 20px;
      margin-right: 20px;
    }

    .apps-item-body {
      margin-top: 20px;

    }

    .apps-item-name {
      margin-bottom: 5px;
      font-size: 12px;

    }


  }

}
</style>
