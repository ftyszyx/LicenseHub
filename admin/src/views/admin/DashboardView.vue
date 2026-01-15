<template>
  <div>
    <h1 class="text-3xl font-bold mb-6">{{ $t('dashboard.title') }}</h1>
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
      <!-- Stat Card 1 -->
      <div class="bg-white p-6 rounded-lg shadow-md">
        <h3 class="text-gray-600 text-sm font-medium">{{ $t('dashboard.total_revenue') }}</h3>
        <p class="text-3xl font-bold">$0</p>
      </div>
      <!-- Stat Card 2 -->
      <div class="bg-white p-6 rounded-lg shadow-md">
        <h3 class="text-gray-600 text-sm font-medium">{{ $t('dashboard.total_orders') }}</h3>
        <p class="text-3xl font-bold">0</p>
      </div>
      <!-- Stat Card 3 -->
      <div class="bg-white p-6 rounded-lg shadow-md">
        <h3 class="text-gray-600 text-sm font-medium">{{ $t('dashboard.total_users') }}</h3>
        <p class="text-3xl font-bold">0</p>
      </div>
      <!-- Stat Card 4 -->
      <div class="bg-white p-6 rounded-lg shadow-md">
        <h3 class="text-gray-600 text-sm font-medium">{{ $t('dashboard.new_orders') }}</h3>
        <p class="text-3xl font-bold">0</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
</script> 