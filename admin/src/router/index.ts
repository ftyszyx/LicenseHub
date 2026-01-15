import { createRouter, createWebHistory } from 'vue-router'
import AdminLayout from '@/layouts/AdminLayout.vue'
import DashboardView from '@/views/admin/DashboardView.vue'
import UserAdminView from '@/views/admin/UserAdminView.vue'
import AppAdminView from '@/views/admin/AppAdminView.vue'
import RegCodesAdminView from '@/views/admin/RegCodesAdminView.vue'
import RoleAdminView from '@/views/admin/RoleAdminView.vue'
import LoginView from '@/views/auth/LoginView.vue'
import RegisterView from '@/views/auth/RegisterView.vue'
import DevicesAdminView from '@/views/admin/DevicesAdminView.vue'
import { useAuthStore } from '@/stores/auth'
import { RouteName, RoutePath } from '@/types'

const router = createRouter({
    history: createWebHistory(import.meta.env.BASE_URL),
    routes: [
        {
            path: '/',
            component: () => import('@/layouts/HomeLayout.vue'),
            children: [
                {
                    path: '',
                    name: RouteName.Home,
                    component: () => import('@/views/HomeView.vue')
                },
                {
                    path: RoutePath.Products,
                    name: RouteName.Products,
                    component: () => import('@/views/ProductsView.vue')
                }
            ]
        },
        {
            path: RoutePath.Login,
            name: RouteName.Login,
            component: LoginView
        },
        {
            path: RoutePath.Register,
            name: RouteName.Register,
            component: RegisterView
        },
        {
            path: RoutePath.Admin,
            component: AdminLayout,
            meta: { requiresAuth: true },
            redirect: RoutePath.AdminDashboard,
            children: [
                {
                    path: RoutePath.AdminDashboard,
                    name: RouteName.AdminDashboard,
                    component: DashboardView
                },
                {
                    path: RoutePath.AdminUsers,
                    name: RouteName.AdminUsers,
                    component: UserAdminView
                },
                {
                    path: RoutePath.AdminRoles,
                    name: RouteName.AdminRoles,
                    component: RoleAdminView
                },
                {
                    path: RoutePath.AdminApps,
                    name: RouteName.AdminApps,
                    component: AppAdminView
                },
                {
                    path: RoutePath.AdminDevices,
                    name: RouteName.AdminDevices,
                    component: DevicesAdminView
                },
                {
                    path: '/admin/reg_codes',
                    name: 'admin_reg_codes',
                    component: RegCodesAdminView
                }
            ]
        }
    ]
})

router.beforeEach((to, _, next) => {
    const authStore = useAuthStore()
    if (to.meta.requiresAuth && !authStore.isAuthenticated) {
        next({ name: 'login' })
    } else {
        next()
    }
})

export default router 