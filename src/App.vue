<template>
  <el-container class="layout-container-demo" style="height: 600px">
    <!-- 侧边栏容器 -->
    <el-aside width="200px">
      <el-scrollbar>

        <el-menu :default-openeds="['1']">
          <el-sub-menu index="1">
            <template #title>
              <el-icon><Setting /></el-icon>设备列表
            </template>
            <el-menu-item-group>
              <template #title>Group 1</template>
              <el-menu-item index="1-1">Option 1</el-menu-item>
              <el-menu-item index="1-2">Option 2</el-menu-item>
            </el-menu-item-group>
            <el-menu-item-group title="Group 2">
              <el-menu-item index="1-3">Option 3</el-menu-item>
            </el-menu-item-group>
            <el-sub-menu index="1-4">
              <template #title>Option4</template>
              <el-menu-item index="1-4-1">Option 4-1</el-menu-item>
            </el-sub-menu>
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
            <el-icon style="margin-right: 8px; margin-top: 1px"
              ><setting
            /></el-icon>
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
          <div class="card">
            <input id="greet-input" v-model="name" placeholder="Enter a name..." />
            <button type="button" @click="greet()">Greet</button>
          </div>
          <p>{{ greetMsg }}</p>

          <Cards />
          <el-table :data="tableData">
            <el-table-column prop="date" label="Date" width="140" />
            <el-table-column prop="name" label="Name" width="120" />
            <el-table-column prop="address" label="Address" />
          </el-table>
        </el-scrollbar>

      </el-main>
      
    </el-container>

  </el-container>
</template>

<script lang="ts" setup>
import  Cards  from './components/cards/Cards.vue'
import { ref } from 'vue'
import { Menu as IconMenu, Message, Setting } from '@element-plus/icons-vue'

import { invoke } from '@tauri-apps/api'

const item = {
  date: '2022-10-10',
  name: 'Tom',
  address: 'No. 189, Grove St, Los Angeles',
}

const tableData = ref(Array.from({ length: 0 }).fill(item))

const greetMsg = ref("");
const name = ref("");

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  greetMsg.value = await invoke("greet", { name: name.value });
}

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
