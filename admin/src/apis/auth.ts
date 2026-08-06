import request from "@/utils/request"
import type { ApiResponse, AuthPayload, AuthResponse, RegisterPayload, StartEmailVerificationInfo, StartEmailVerificationPayload, UserModel, VerifyEmailCodeInfo } from "@/types"
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

export const startEmailVerification = async (payload: StartEmailVerificationPayload) => {
    const response = await request.post('/auth/email-verifications', payload) as ApiResponse<StartEmailVerificationInfo>
    return response.data
}

export const verifyEmailCode = async (challengeId: string, code: string) => {
    const response = await request.post(`/auth/email-verifications/${challengeId}/verify`, { code }) as ApiResponse<VerifyEmailCodeInfo>
    return response.data
}

export const changeMyPassword = async (payload: ChangePasswordPayload) => {
    const response = await request.post('/admin/me/password', payload) as ApiResponse<boolean>
    return response.data
}
