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
import KanbanColumn from "../components/KanbanColumn.vue";

const columns = [
  {
    title: "To Do",
    items: [
      { id: 1, title: "Task 1", description: "Description for Task 1" },
      { id: 2, title: "Task 2", description: "Description for Task 2" },
    ],
  },
  {
    title: "In Progress",
    items: [{ id: 3, title: "Task 3", description: "Description for Task 3" }],
  },
  {
    title: "Done",
    items: generate(4, 20),
  },
];

function generate(start: number, total: number) {
  const items = [];
  for (let i = start; i < start + total; i++) {
    items.push({
      id: i,
      title: `Task ${i}`,
      description: `Description for Task ${i}`,
    });
  }
  return items;
}
</script>

<template>
  <div class="flex justify-center p-6 gap-4">
    <div v-for="column in columns" :key="column.title" class="flex-1 max-w-80">
      <KanbanColumn :title="column.title" :items="column.items" />
    </div>
  </div>
</template>
