import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'dashboard',
      component: () => import('@/views/DashboardView.vue'),
    },
    {
      path: '/agents',
      name: 'agents',
      component: () => import('@/views/AgentsView.vue'),
    },
    {
      path: '/providers',
      name: 'providers',
      component: () => import('@/views/ProvidersView.vue'),
    },
    {
      path: '/profiles',
      name: 'profiles',
      component: () => import('@/views/ProfilesView.vue'),
    },
    {
      path: '/profiles/:alias',
      name: 'profile-detail',
      component: () => import('@/views/ProfileDetailView.vue'),
    },
    {
      path: '/proxy',
      name: 'proxy',
      component: () => import('@/views/ProxyView.vue'),
    },
    {
      path: '/stats',
      name: 'stats',
      component: () => import('@/views/StatsView.vue'),
    },
    {
      path: '/usage',
      name: 'usage',
      component: () => import('@/views/UsageView.vue'),
    },
    {
      path: '/terminal',
      name: 'terminal',
      component: () => import('@/views/TerminalView.vue'),
    },
    {
      path: '/terminal/:sessionId',
      name: 'terminal-session',
      component: () => import('@/views/TerminalView.vue'),
    },
  ],
})

export default router
