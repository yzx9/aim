<!--
Copyright 2025 Zexin Yuan

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

   http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
-->

<script lang="ts" setup>
import { defineProps } from "vue";
import KanbanCard from "./KanbanCard.vue";

const props = defineProps<{
  title: string;
  items: { id: number; title: string; description: string }[];
}>();
</script>

<template>
  <section
    class="flex flex-col p-2 w-full max-w-sm border border-gray-100 rounded-md bg-white"
  >
    <h2 class="m-2 text-lg font-semibold">{{ title }}</h2>

    <div class="p-2 overflow-y-auto">
      <KanbanCard v-for="item in props.items" :key="item.id" :item="item" />
    </div>
  </section>
</template>
