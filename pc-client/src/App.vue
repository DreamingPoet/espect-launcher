<script setup lang="ts">
import { appWindow } from '@tauri-apps/api/window'
import { onMounted, onUnmounted, reactive } from 'vue'
import { invoke } from "@tauri-apps/api";
import { SemiSelect, CloseBold, FolderOpened } from '@element-plus/icons-vue'

let socket:any;

// webSocket连接成功
const socket_onopen = function (e:any) {
  data.connected = true;
  console.log("[open] Connection established");
  get_local_data();
};

// 收到消息, 只接收 ClientFunc 类型的数据
const socket_onmessage = function (event:any) {
  console.log(`[message] Data received from server: ${event.data}`);
  let client_data = eval("(" + event.data + ")");

 if (event.data == "get_client_data") {
    get_local_data();

  } else if (client_data.func_name == "start_app") {
    console.log(client_data.data)
    
    start_app(client_data.data);
  } else {
    // console.log(event.data)
  }

};

// webSocket关闭
const socket_onclose = function (event:any) {
  data.connected = false;
  if (event.wasClean) {
    console.log(`[close] Connection closed, code=${event.code} reason=${event.reason}`);
  } else {
    console.log('[close] Connection died');
  }
};

// webSocket 错误
const socket_onerror = function (error:any) {
  data.connected = false;
  console.log(error);
};

// ======== data start ========
const data = reactive({
  host: '',
  connected: false,
  checkTimer: 1,
  state: "waiting ...",
  debug:"start"

});
// ======== data end ========


// ======== to tauri start ========
const sayhello = function () {
  // socket.send('hello');
  get_local_data();
};


const reconnect = function () {
  invoke("reconnect").then(
    (data) => {
      // socket.send(data);
    }
  );
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

// 获取 本地数据 机器参数 + apps
const get_local_data = () => {
  // 从后台获取数据
  invoke("get_local_data").then(
    (data) => {
      socket.send(data);
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

// 启动本地 app 目录
const start_app = (app:string) => {
  // 从后台获取数据
  invoke("start_app", { app: app }).then(
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

const connect_websocket = () => {
  data.debug = "connect_websocket"+ data.host;
  // 连接到websocket服务器
  if (socket != null) {
    socket.close()
    data.debug = "close";
  }
  socket = new WebSocket("ws://" + data.host + "/ws");
  data.debug = socket;
  socket.onopen = socket_onopen;
  socket.onmessage = socket_onmessage;
  socket.onclose = socket_onclose;
  socket.onerror = socket_onerror;
  //data.debug = "new WebSocket" + data.host;
}

onMounted(() => {
  data.debug = "mounted";
  get_saved_host();
  data.debug = "getsaved";
  data.checkTimer = setInterval(() => {
    data.debug = "start tick";
    if (!data.connected) {
      // connect_websocket();
      data.state = "try connecting to " + data.host + " ...";
      data.debug = "try connecting ...";
      console.log(data.state);
    } else {
      data.state = "connected to " + data.host + "!";
      socket.send("tick");
      data.debug = "connected to ...";
      // console.log(data.state);
    }
  }, 5000);


});

onUnmounted(() => {
  clear();
});




// ===== window title =======
const minimize = () => {
  appWindow.minimize();
}

const close = () => {
  appWindow.hide();
}
// ===== window title =======

</script>

<template>

  <div data-tauri-drag-region class="titlebar">
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
      <h5>{{data.debug}}</h5>
    </div>

    <div style="text-align:center; margin-top: 30px;">
      <el-button type="primary" @click="open_app_folder" plain :icon="FolderOpened" >打开目录</el-button>
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
