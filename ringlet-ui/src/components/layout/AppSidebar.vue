<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import { isTauri } from '@/api/config'

const route = useRoute()

const baseNavItems = [
  { name: 'Workspaces', path: '/', icon: 'folder' },
  { name: 'Dashboard', path: '/dashboard', icon: 'home' },
  { name: 'Agents', path: '/agents', icon: 'cpu' },
  { name: 'Providers', path: '/providers', icon: 'cloud' },
  { name: 'Profiles', path: '/profiles', icon: 'user' },
  { name: 'Terminal', path: '/terminal', icon: 'terminal' },
  { name: 'Proxy', path: '/proxy', icon: 'shuffle' },
  { name: 'Stats', path: '/stats', icon: 'bar-chart' },
  { name: 'Usage', path: '/usage', icon: 'currency' },
]

const navItems = computed(() => {
  if (isTauri()) {
    return [...baseNavItems, { name: 'Settings', path: '/settings', icon: 'settings' }]
  }
  return baseNavItems
})

function isActive(path: string): boolean {
  if (path === '/') {
    return route.path === '/' || route.path.startsWith('/workspace')
  }
  return route.path.startsWith(path)
}
</script>

<template>
  <aside class="w-56 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 min-h-[calc(100vh-65px)]">
    <nav class="p-4 space-y-1">
      <RouterLink
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        :class="[
          'flex items-center px-4 py-2.5 rounded-lg text-sm font-medium transition-colors',
          isActive(item.path)
            ? 'bg-blue-50 dark:bg-blue-900/50 text-blue-700 dark:text-blue-300'
            : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
        ]"
      >
        <svg class="w-5 h-5 mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <template v-if="item.icon === 'folder'">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </template>
          <template v-else-if="item.icon === 'home'">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
          </template>
          <template v-else-if="item.icon === 'cpu'">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
          </template>
          <template v-else-if="item.icon === 'cloud'">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z" />
          </template>
          <template v-else-if="item.icon === 'user'">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
          </template>
          <template v-else-if="item.icon === 'terminal'">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
          </template>
          <template v-else-if="item.icon === 'shuffle'">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
          </template>
          <template v-else-if="item.icon === 'bar-chart'">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
          </template>
          <template v-else-if="item.icon === 'currency'">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </template>
          <template v-else-if="item.icon === 'settings'">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </template>
        </svg>
        {{ item.name }}
      </RouterLink>
    </nav>
  </aside>
</template>
