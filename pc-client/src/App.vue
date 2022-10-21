<script setup lang="ts">
import { invoke } from "@tauri-apps/api";
import { appWindow } from '@tauri-apps/api/window'
import { listen, Event } from "@tauri-apps/api/event"
import { onMounted, onUnmounted, reactive } from 'vue'
import { SemiSelect, CloseBold, FolderOpened } from '@element-plus/icons-vue'

// ======== data start ========
const data = reactive({
  host: '',
  connected: false,
  checkTimer: 1,
  state: "waiting ...",

});
// ======== data end ========


// ======== to tauri start ========


const reconnect = function () {
  invoke("reconnect", { host: data.host });
};


// 获取保存的host
const get_saved_host = () => {
  invoke("get_saved_host").then(
    (host) => {
      data.host = host as string;
      // connect_websocket();
    }
  );
};

// 打开本地 app 目录
const open_app_folder = () => {
  // 从后台获取数据
  invoke("open_app_folder").then(
    (data) => {
    }
  );
};

// ======== to tauri end ========

const clear = () => {
  if (data.checkTimer) {
    clearInterval(data.checkTimer);
  }
};


onMounted(() => {
  get_saved_host();
});

onUnmounted(() => {
  clear();
});

// 监听来自后端的事件
// ======== from tauri start =========
listen("keep-alive", function (data: Event<any>) {
  console.log(data.payload);
});

// 获取来自客户端的数据之后, 更新 UI
listen("check_connect_state", function (event_data: Event<any>) {
  if (event_data.payload) {
    data.state = "connected to " + data.host + "!";
  } else {
    data.state = "try connecting to " + data.host + " ...";
  }
});


// ======== from tauri end =========




// ===== window title start =======
const minimize = () => {
  appWindow.minimize();
}

const close = () => {
  appWindow.hide();
}
// ===== window title end =======

</script>

<template>

  <div data-tauri-drag-region class="titlebar">
    <div class="buttons">

      <div @click="minimize()" class="titlebar-button" id="titlebar-minimize">
        <el-icon>
          <SemiSelect />
        </el-icon>

      </div>
      <div @click="close()" class="titlebar-button-close" id="titlebar-close">
        <el-icon>
          <CloseBold />
        </el-icon>
      </div>

    </div>

  </div>

  <div class="window-main">
    <el-row :gutter="24">
      <el-col :span="12" :offset="3">
        <el-input v-model="data.host" placeholder="Please input" />
      </el-col>
      <el-col :span="6">
        <el-button type="primary" @click="reconnect" plain>重连</el-button>
      </el-col>
    </el-row>

    <div style="text-align:center; margin-top: 10px;">
      <h5>{{data.state}}</h5>
    </div>

    <div style="text-align:center; margin-top: 30px;">
      <el-button type="primary" @click="open_app_folder" plain :icon="FolderOpened">打开目录</el-button>
    </div>

  </div>
</template>

<style scoped>
.titlebar {
  height: 30px;
  background: #eeeeee;
  user-select: none;
  display: flex;
  justify-content: flex-end;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
}

.buttons {

  justify-content: right;

}

.titlebar-text {
  font-size: 15px;
  text-align: center;
  vertical-align: middle;
  justify-content: left;
  margin-left: 5px;


}

.titlebar-button,
.titlebar-button-close {
  display: inline-flex;
  justify-content: center;
  align-items: center;
  width: 30px;
  height: 30px;
}

.titlebar-button:hover {
  background: #c4c4c4;
}

.titlebar-button-close:hover {
  background: #e03a3a;
}


.window-main {
  margin-top: 80px;
}
</style>
