
export interface AuthPayload {
    username: string;
    password: string;
}

export interface AuthResponse {
    token: string;
}

export interface RegisterPayload {
    username: string;
    email: string;
    password: string;
    verification_token: string;
}

export interface StartEmailVerificationPayload {
    email: string
}

export interface StartEmailVerificationInfo {
    challenge_id: string
    expires_in_seconds: number
    resend_after_seconds: number
}

export interface VerifyEmailCodeInfo {
    verification_token: string
    expires_in_seconds: number
}
