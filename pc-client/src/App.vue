<script setup lang="ts">
import { appWindow } from '@tauri-apps/api/window'
import { onMounted, onUnmounted, reactive } from 'vue'
import { invoke } from "@tauri-apps/api";

let socket = null;

const socket_onopen = function(e) {
  data.connected = true;
  console.log("[open] Connection established");
  socket.send("start_connection");
};

const socket_onmessage = function(event) {
  console.log(`[message] Data received from server: ${event.data}`);
  if (event.data == "file_changed") {
    window.location.reload();
  } else if (event.data == "get_client_data") {
    get_local_data();

  } else if (event.data == "xxx") {

  }

};

const socket_onclose = function(event) {
  data.connected = false;
  if (event.wasClean) {
    console.log(`[close] Connection closed, code=${event.code} reason=${event.reason}`);
  } else {
    console.log('[close] Connection died');
  }
};

const socket_onerror = function(error) {
  data.connected = false;
  console.log(error);
};

const data = reactive({
  host: '',
  connected: false,
  checkTimer: null,
  state: "waiting ..."

});



// ============================================= to tauri =============================================
const sayhello = function () {
  socket.send('hello');
};

// 获取保存的host
const get_saved_host = () => {
  invoke("get_saved_host").then(
    // (username) => (data.username = username)
    (host) => {
      data.host = host;
      connect_websocket();
    }
  );
};

// 获取 本地数据 机器参数 + apps
const get_local_data = () => {
  invoke("get_local_data").then(
    (data) => {
      socket.send(data);
    }
  );
};


// ============================================= to tauri =============================================



const clear = () => {
  if (data.checkTimer) {
    clearInterval(data.checkTimer);
  }
};

const connect_websocket = () => {
      // 连接到websocket服务器
      if (socket != null) {
        socket.close()     
      }
      socket = new WebSocket("ws://" + data.host + "/ws");
      socket.onopen = socket_onopen;
      socket.onmessage = socket_onmessage;
      socket.onclose = socket_onclose;
      socket.onerror = socket_onerror;
}

onMounted(() => {

  get_saved_host();
  data.playTimer = setInterval(() => {
    if (!data.connected) {
      connect_websocket();
      data.state = "try connecting to " + data.host + " ...";
      console.log(data.state);
    }else{
      data.state = "connected to " + data.host + "!";
      console.log(data.state);
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
      <img src="https://api.iconify.design/mdi:window-minimize.svg" alt="minimize" />
    </div>
    <div @click="close()" class="titlebar-button" id="titlebar-close">
      <img src="https://api.iconify.design/mdi:close.svg" alt="close" />
    </div>
  </div>

  <div class="window-main">
    <el-input v-model="data.host" placeholder="Please input" />
    <el-button type="primary" @click="sayhello">Send hello </el-button>
    <el-button type="primary" @click="get_saved_host">get saved host</el-button>
    <h5>{{data.state}}</h5>
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

.titlebar-button {
  display: inline-flex;
  justify-content: center;
  align-items: center;
  width: 30px;
  height: 30px;
}

.titlebar-button:hover {
  background: #c4c4c4;
}

.window-main {
  margin-top: 30px;
}
</style>