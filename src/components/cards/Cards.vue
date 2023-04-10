<template>
  <el-row>
    <el-col v-for="(o, index) in 2" :key="o" :span="8" :offset="index > 0 ? 2 : 0">
      <el-card :body-style="{ padding: '0px' }">
        <img src="../../assets/exe.png" class="image" />
        <div style="padding: 14px">
          <span>App</span>
          <div class="bottom">

            <el-button text class="button">Operating</el-button>
          </div>
          <el-button type="primary">启动</el-button>
        </div>
      </el-card>
    </el-col>
  </el-row>
</template>
  
<script lang="ts" setup>
import { ref } from 'vue'


</script>
  
<style>
.time {
  font-size: 12px;
  color: #999;
}

.bottom {
  margin-top: 13px;
  line-height: 12px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.button {
  padding: 0;
  min-height: auto;
}

.image {
  width: 120px;
  display: block;
}
</style>