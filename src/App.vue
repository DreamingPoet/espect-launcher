<template>
  <div class="common-layout">
    <el-container class="layout-container-demo" style="height: 580px">
      <el-header>

      </el-header>

      <el-container>
        <el-aside width="200px">
          <el-scrollbar>
            <el-menu>
              <el-menu-item index="1-1" v-for="[key, value] in data.clients" @click="call_option(key)">Option {{
              value.id }}</el-menu-item>
            </el-menu>
          </el-scrollbar>
        </el-aside>

        <el-main>

          <!-- 内部容器 main-->
          <el-scrollbar>
            <div v-if="data.clients.get(data.current_client_id)">
              <el-row>
                <div>{{data.clients.get(data.current_client_id).name}}</div>
                <div>{{data.clients.get(data.current_client_id).ip}}</div>

              </el-row>


              <el-row>
       

              </el-row>

              <el-row>
                <el-col v-for="(item, index) in data.clients.get(data.current_client_id).apps" :span="6" :offset="1">
                  <el-card :body-style="{ padding: '0px' }">
                    <img src="./assets/vue.svg" class="image" />
                    <div style="padding: 14px">
                      <div>{{item.name}}</div>
                      <el-button v-if="item.state" type="danger" @click="start_app(item.name, false)">关闭</el-button>

                      <el-button v-if="!item.state" type="primary" @click="start_app(item.folder + '/' +item.name, true)">启动</el-button>

                    </div>
                  </el-card>
                </el-col>
              </el-row>

            </div>

          </el-scrollbar>

        </el-main>

      </el-container>
    </el-container>
  </div>
</template>

<script lang="ts" setup>
import Cards from './components/cards/Cards.vue'
import { onMounted, onUnmounted, reactive } from 'vue'
import { Menu as IconMenu, Message, Setting } from '@element-plus/icons-vue'

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

  // console.log(data.clients.get(data.current_client_id).apps)
  for(let i of update_data.apps) {
    if( i.data == "true"){
      for( let j of data.clients.get(data.current_client_id).apps ) {
        if(j.name == i.name) {
          console.log(j.name + "set true")
          j.state = true;
        }
      }
    }else {
      for( let j of data.clients.get(data.current_client_id).apps ) {
        if(j.name == i.name) {
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
  state:boolean;
  // 构造函数
  constructor(name: string, folder: string, dscrpt: string, icon: string, state:boolean) {
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

//  页面中绑定的数据

const data = reactive({
  clients: new Map(),
  current_client_id: -1,
  update_client_timer: -1,

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
    invoke("start_app", { id: data.clients.get(data.current_client_id).id, start:start,  app: app_path });
  }
};


const clear = () => {
  if (data.update_client_timer) {
    clearInterval(data.update_client_timer);
  }
};


onMounted(() => {

  data.update_client_timer = setInterval(() => {
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

<style scoped>
.el-sub-menu {

  background-color: var(--el-color-primary-light-7);
  color: var(--el-text-color-primary);
}

.el-menu-item {

  background-color: var(--el-color-primary-light-9);
  color: var(--el-text-color-primary);
}

.layout-container-demo .el-header {
  position: relative;
  background-color: var(--el-color-primary-light-7);
  color: var(--el-text-color-primary);
}

.layout-container-demo .el-aside {
  color: var(--el-text-color-primary);
  background: var(--el-color-primary-light-8);
}

.layout-container-demo .el-menu {
  border-right: none;
}

.layout-container-demo .el-main {
  padding: 0;
}

.layout-container-demo .toolbar {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  right: 20px;
}
</style>
