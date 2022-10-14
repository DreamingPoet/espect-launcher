<script setup lang="ts">

const socket = new WebSocket("ws://127.0.0.1:3000/ws");

socket.onopen = function(e) {
  console.log("[open] Connection established");
  console.log("Sending to server");
  socket.send("Sending to server");
  socket.send("start_connection");
};

socket.onmessage = function(event) {
  console.log(`[message] Data received from server: ${event.data}`);
  socket.send('hello');
  if (event.data == "file_changed") {
    window.location.reload();
  }
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
