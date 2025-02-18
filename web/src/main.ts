// Copyright 2025 Zexin Yuan
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import { createApp } from "vue";
import { createRouter, createWebHistory } from "vue-router";
import { createAPIs } from "./apis";
import App from "./App.vue";
import Home from "./views/Home.vue";
import Project from "./views/Project.vue";
import "./style.css";

const routes = [
  { path: "/", component: Home },
  { path: "/project", component: Project },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

const apis = createAPIs(import.meta.env.VITE_URL_BASE);

createApp(App).use(router).use(apis).mount("#app");
