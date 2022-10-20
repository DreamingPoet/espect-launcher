<script lang="ts" setup>
import Cards from './components/cards/Cards.vue'
import { onMounted, onUnmounted, reactive } from 'vue'
import { Menu, Message, Setting, Platform, Opportunity } from '@element-plus/icons-vue'

import { invoke } from '@tauri-apps/api'
import { listen, Event } from "@tauri-apps/api/event"


// 监听来自后端的事件
// ======== from tauri start =========
listen("keep-alive", function (data: Event<any>) {
  console.log(data.payload);
});

// 获取来自客户端的数据之后, 更新 UI
listen("on_get_client_data", function (client_data: Event<any>) {
  console.log(client_data.payload);
  let temp_data: ClientData = eval("(" + client_data.payload + ")");
  data.clients.set(temp_data.id, temp_data);
});


// 客户端断开连接
listen("remove_client_data", function (client_data: Event<any>) {
  console.log("remove_client_data id = " + client_data.payload);
  data.clients.delete(client_data.payload);
});

// 接收来自客户端的更新信息
listen("on_update_client", function (client_data: Event<any>) {
  let update_data: ClientUpdateData = eval("(" + client_data.payload + ")");
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
});
// ======== from tauri end =========


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

var update_client_timer:any;
//  页面中绑定的数据
let client_states: ClientStateData[] = [];
const data = reactive({
  clients: new Map(),
  current_client_id: -1,
  menu_activeIndex: 1,
  client_states: client_states,

});

// ======== data end ========


// ======== web func start ========

const call_option = function (index: number) {
  console.log("call index = " + index)
  data.current_client_id = index;
};

// 启动app
const start_app = (app_path: string, start: boolean) => {
  if (data.clients.get(data.current_client_id) != undefined) {
    invoke("start_app", { id: data.clients.get(data.current_client_id).id, start: start, app: app_path });
  }
};


const clear = () => {
  if (update_client_timer) {
    clearInterval(update_client_timer);
  }
};

// 测试数据
const test_client = new Map([
  [1,
    {
      id: 1,
      name: "MSFD-S123",
      ip: "127.0.0.1",
      b_admin: false,
      client_op: [
        {
          name: "Steam VR",
          dscrpt: "SDFDSFDS",
        },
        {
          name: "Windows MR",
          dscrpt: "SDFDSFDS",
        }
      ],
      apps: [
        {
          name: "FilmingExperiment",
          folder: "string", // 文件夹名称
          dscrpt: "string",
          icon: "",
          state: false,
        },
        {
          name: "PTPlatform",
          folder: "string", // 文件夹名称
          dscrpt: "string",
          icon: "",
          state: true,
        },
      ],
    }

  ],
  [2,
    {
      id: 2,
      name: "MSFD-S123",
      ip: "127.0.0.1",
      b_admin: false,
      client_op: [
        {
          name: "Steam VR",
          dscrpt: "SDFDSFDS",
        },
        {
          name: "Windows MR",
          dscrpt: "SDFDSFDS",
        }
      ],
      apps: [
        {
          name: "FilmingExperiment",
          folder: "string", // 文件夹名称
          dscrpt: "string",
          icon: "",
          state: true,
        },
        {
          name: "PTPlatform",
          folder: "string", // 文件夹名称
          dscrpt: "string",
          icon: "",
          state: true,
        },
      ],
    }

  ]
]);


onMounted(() => {

  // 测试代码
  // data.clients = test_client;
  // data.client_states = [
  //   {
  //     name: "CPU",
  //     data: "90%",
  //   },
  //   {
  //     name: "FPS",
  //     data: "50",
  //   },
  //   {
  //     name: "Steam VR",
  //     data: "true",
  //   }
  // ];

  update_client_timer = setInterval(() => {
    update_client();
  }, 2000);


});

// 更新客户端信息
const update_client = () => {
  if (data.current_client_id < 0) return;
  invoke("update_client", { id: data.current_client_id });
};


onUnmounted(() => {
  clear();
});

// ======== web func end ========

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
              <el-icon>
                <Platform />
              </el-icon>设备 {{
              value.id }}
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

            <div class="client-state">

              <div class="client-state-item" v-for="item in data.client_states"> 状态 <el-icon>
                  <Opportunity />
                </el-icon>
              </div>

            </div>

            <div class="client-option">

              <el-button v-for="item in data.clients.get(data.current_client_id).client_op" type="primary" plain>
                {{item.name}}</el-button>

            </div>

            <div class="client-apps">

              <el-card class="client-apps-item" v-for="(item, index) in data.clients.get(data.current_client_id).apps" body-style="padding:15px">
                <img src="./assets/vue.svg" class="image" />
                <div class="apps-item-body">
                  <div class="apps-item-name">{{item.name}}</div>
                  <el-button v-if="item.state" type="danger" @click="start_app(item.name, false)">关闭</el-button>

                  <el-button v-if="!item.state" type="primary" @click="start_app(item.folder + '/' +item.name, true)">
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
    width: 200px;
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
      width: 360px;
    }

    .el-icon {
      color: #00ff00;
    }
  }

  .client-option{
    display: flex;
    margin-top: 20px;
  }

  .client-apps {
    display: flex;
    flex-wrap: wrap;
 

    .client-apps-item {
      width: 260px;
      margin-top: 20px;
      margin-right: 20px;
    }

    .apps-item-body{
      margin-top: 20px;

    }

    .apps-item-name{
      margin-bottom: 5px;

    }


  }

}
</style>
