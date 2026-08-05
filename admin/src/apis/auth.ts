import request from "@/utils/request"
import type { ApiResponse, AuthPayload, AuthResponse, RegisterPayload, UserModel } from "@/types"
import type { ChangePasswordPayload } from "@/types"


export const sentLogin = async (payload: AuthPayload)=> {
    const response = await request.post('/login', payload) as ApiResponse<AuthResponse>
    return response.data
}

export const sentRegister = async (payload: RegisterPayload) => {
    const response = await request.post('/register', payload) as ApiResponse<AuthResponse>
    return response.data
}


export const sentGetUserInfo = async () => {
    const response = await request.get('/admin/me') as ApiResponse<UserModel>
    return response.data
}

export const changeMyPassword = async (payload: ChangePasswordPayload) => {
    const response = await request.post('/admin/me/password', payload) as ApiResponse<boolean>
    return response.data
}
