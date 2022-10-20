<script setup lang="ts">
import { onMounted, onUnmounted, reactive } from 'vue'

onMounted(() => {
  // http://localhost:3001/?server=192.168.0.0.1:3000&port=5000
  let server = getQueryVariable("server")
  let port = getQueryVariable("port")
  console.log(server);
  console.log(port);
});

onUnmounted(() => {
});
const socket = new WebSocket("ws://127.0.0.1:3000/ws");


function getQueryVariable(variable:string)
{
       var query = window.location.search.substring(1);
       var vars = query.split("&");
       for (var i=0;i<vars.length;i++) {
               var pair = vars[i].split("=");
               if(pair[0] == variable){return pair[1];}
       }
       return(false);
}



socket.onopen = function(e) {
  console.log("[open] Connection established");
  console.log("Sending to server");
  socket.send("Sending to server");
  socket.send("start_connection");
};

socket.onmessage = function(event) {
  console.log(`[message] Data received from server: ${event.data}`);
  let server = getQueryVariable("server")
  console.log(server);

};

socket.onclose = function(event) {
  if (event.wasClean) {
    console.log(`[close] Connection closed, code=${event.code} reason=${event.reason}`);
  } else {
    console.log('[close] Connection died');
  }
};

socket.onerror = function(error) {
  console.log(error)
};

const sayhello = function() {
  socket.send('hello');
};



</script>

<template>
<el-button type="primary" @click="sayhello" >Send hello  </el-button>
</template>

<style scoped>

</style>
