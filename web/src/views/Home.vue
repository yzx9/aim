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
    class="home flex justify-center min-h-screen bg-gradient-to-r from-purple-300 to-blue-200"
  >
    <div class="home_title mt-32 flex flex-col items-center gap-6">
      <img class="h-40" src="/logo.svg" />

      <h1 class="text-6xl font-bold text-white">AIM</h1>

      <h2 class="text-2xl font-bold text-white">An Information Management</h2>
    </div>
  </div>
</template>

<style>
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
