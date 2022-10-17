<template>
  <el-container class="layout-container-demo" style="height: 580px">
    <!-- 侧边栏容器 -->
    <el-aside width="200px">
      <el-scrollbar>

        <el-menu :default-openeds="['1']">

          <el-sub-menu index="1">

            <template #title>
              <el-icon>
                <Setting />
              </el-icon>设备列表
            </template>

            <el-menu-item-group>

              <el-menu-item index="1-1" v-for="(item, index) in data.clients" @click="call_option(index)">Option {{
              item.id }}</el-menu-item>

            </el-menu-item-group>


          </el-sub-menu>
        </el-menu>

      </el-scrollbar>
    </el-aside>

    <!-- 内部容器 -->
    <el-container>
      <!-- 内部容器 header-->
      <el-header style="text-align: right; font-size: 12px">
        <div class="toolbar">
          <el-dropdown>
            <el-icon style="margin-right: 8px; margin-top: 1px">
              <setting />
            </el-icon>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item>View</el-dropdown-item>
                <el-dropdown-item>Add</el-dropdown-item>
                <el-dropdown-item>Delete</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
          <span>Espect</span>
        </div>
      </el-header>

      <!-- 内部容器 main-->
      <el-main>

        <!-- 内部容器 main-->
        <el-scrollbar>

          <el-row>
            <el-col v-for="(item, index) in data.current_client.apps" :span="6" :offset="1">
              <el-card :body-style="{ padding: '0px' }">
                <img src="./assets/vue.svg" class="image" />
                <div style="padding: 14px">
                  <div>{{item.dscrpt}}</div>

  
                    <el-button type="primary" @click="start_app(item.name)">启动</el-button>

                </div>
              </el-card>
            </el-col>
          </el-row>


        </el-scrollbar>

      </el-main>

    </el-container>

  </el-container>
</template>

<script lang="ts" setup>
import Cards from './components/cards/Cards.vue'
import Client from './components/Client.vue'
import { reactive, ref } from 'vue'
import { Menu as IconMenu, Message, Setting } from '@element-plus/icons-vue'

import { invoke } from '@tauri-apps/api'
import { listen, Event } from "@tauri-apps/api/event"


// const item = {
//   date: '2022-10-10',
//   name: 'Tom',
//   address: 'No. 189, Grove St, Los Angeles',
// }

// const tableData = ref(Array.from({ length: 0 }).fill(item))


// 监听来自后端的事件
// ======== from tauri start =========
listen("keep-alive", function (data: Event<any>) {
  console.log(data.payload);
});

// 获取来自客户端的数据之后, 更新 UI
listen("on_get_client_data", function (client_data: Event<any>) {
  console.log(client_data.payload);
  data.clients.push(eval("(" + client_data.payload + ")"));
});

// ======== from tauri end =========



// ======== data start ========

const test = [
  {
    "id": 1,
    "name": "背包1",
    "ip": "127.0.0.1",
    "b_admin": false,
    "client_op": [
      {
        "name": "Start SteamVR",
        "dscrpt": ""
      },
      {
        "name": "Power Down",
        "dscrpt": ""
      }
    ],
    "apps": [
      {
        "name": "F:\\MyWorkSpace\\espect-launcher\\pc-client\\src-tauri\\apps\\launcher\\espect-launcher-client1.exe",
        "dscrpt": "client1",
        "icon": ""
      },
      {
        "name": "F:\\MyWorkSpace\\espect-launcher\\pc-client\\src-tauri\\apps\\launcher2\\launcher-client2.exe",
        "dscrpt": "client2",
        "icon": ""
      }
      ,
      {
        "name": "F:\\MyWorkSpace\\espect-launcher\\pc-client\\src-tauri\\apps\\launcher2\\launcher-client6.exe",
        "dscrpt": "client6",
        "icon": ""
      }
    ]
  }
  ,

  {
    "id": 2,
    "name": "背包2",
    "ip": "127.0.0.1",
    "b_admin": false,
    "client_op": [
      {
        "name": "Start SteamVR",
        "dscrpt": ""
      },
      {
        "name": "Power Down",
        "dscrpt": ""
      }
    ],
    "apps": [
      {
        "name": "F:\\MyWorkSpace\\espect-launcher\\pc-client\\src-tauri\\apps\\launcher\\espect-launcher-client3.exe",
        "dscrpt": "client3",
        "icon": ""
      },
      {
        "name": "F:\\MyWorkSpace\\espect-launcher\\pc-client\\src-tauri\\apps\\launcher2\\launcher-client4.exe",
        "dscrpt": "client4",
        "icon": ""
      }
    ]
  }

]


const data = reactive({
  clients: [{id:"", apps:[]}],
  current_client:{id:"", apps:[{dscrpt:"",name:""}]},

});
// ======== data end ========


// ======== web func start ========

const call_option = function (index: number) {
  console.log("call index = " + index)
  data.current_client = data.clients[index];

  console.log(data.current_client.apps)


};

// 获取保存的host
const start_app = (app_path:string) => {
  invoke("start_app", { index: data.current_client.id, app:app_path  }).then(
    (host) => {

    }
  );
};

// ======== web func end ========



</script>

<style scoped>
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
