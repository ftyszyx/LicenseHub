import type { ListParamsReq } from './api'

export interface AppModel {
    id: number
    name: string
    app_id: string
    app_vername: string
    app_vercode: number
    app_download_url: string
    app_res_url: string
    app_update_info?: string | null
    code_type: number
    app_valid_key: string
    trial_days: number
    trial_num: number
    sort_order: number
    manifest_extra?: Record<string, unknown> | null
    manifest_urls?: AppManifestUrl[]
    status: number
    created_at: string
    updated_at: string
    deleted_at?: string | null
}

export type ListAppsParams = {
    id?: number
    app_id?: string
    name?: string
} & ListParamsReq

export interface AddAppReq {
    name: string
    app_id: string
    app_vername: string
    app_vercode: number
    app_download_url: string
    app_res_url: string
    app_update_info?: string | null
    code_type?: number | null
    app_valid_key?: string | null
    trial_days?: number | null
    trial_num?: number | null
    sort_order: number
    manifest_extra?: Record<string, unknown>
    status: number
}

export interface AppManifestUrl {
    channel_id: number
    channel_name: string
    provider: string
    public_url: string
    object_key: string
    synced_at: string
}

export interface UpdateAppReq {
    name?: string
    app_id?: string
    app_vername?: string
    app_vercode?: number
    app_download_url?: string
    app_res_url?: string
    app_update_info?: string | null
    code_type?: number | null
    app_valid_key?: string | null
    trial_days?: number | null
    trial_num?: number | null
    sort_order?: number
    manifest_extra?: Record<string, unknown>
    status?: number
}
