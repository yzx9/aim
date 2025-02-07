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

const props = defineProps<{
  item: { title: string; description: string };
}>();
</script>

<template>
  <div class="bg-white rounded-md border border-gray-200 p-3 mb-2">
    <h3 class="text-md font-medium">{{ props.item.title }}</h3>

    <p class="text-sm text-gray-500">{{ props.item.description }}</p>
  </div>
</template>
