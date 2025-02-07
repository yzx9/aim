<!--
Copyright 2025 Zexin Yuan

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

   http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the specific language governing permissions and
limitations under the License.
-->

<template>
  <div
    class="home flex justify-center items-center min-h-screen bg-gradient-to-r from-purple-500 to-blue-500"
  >
    <h1 class="home-title text-5xl font-bold text-white text-center">
      AIM - An Information Management system
    </h1>
  </div>
</template>

<style scoped>
.home .home_title {
  animation: fadeIn 2s ease-in-out;
}

@keyframes fadeIn {
  0% {
    opacity: 0;
    transform: translateY(-20px);
  }
  100% {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
