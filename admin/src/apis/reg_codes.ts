import request from '@/utils/request'
import type { ApiResponse, PagingResponse } from '@/types'
import type {
  ListRegCodesParams,
  RegCodeModel,
  BatchCreateRegCodesReq,
  RegCodeStatus,
  BindRegCodeReq,
  CheckRegDeviceReq,
  RegCodeBindCheckResp,
  UseCountReq,
  UseCountResp,
} from '@/types/reg_codes'

export const fetchRegCodes = async (params: ListRegCodesParams = {}) => {
  const response = await request.get('/admin/reg_codes/list', { params }) as ApiResponse<PagingResponse<RegCodeModel>>
  return response.data
}

export const deleteRegCode = async (id: number) => {
  const response = await request.delete(`/admin/reg_codes/${id}`) as ApiResponse<void>
  return response.data
}

export const updateRegCodeStatus = async (id: number, status: RegCodeStatus) => {
  const response = await request.put(`/admin/reg_codes/${id}/status`, { status }) as ApiResponse<RegCodeModel>
  return response.data
}

export const batchCreateRegCodes = async (payload: BatchCreateRegCodesReq) => {
  // call create single repeatedly on server? For now assume server has add endpoint; we'll loop client-side as a simple approach.
  const items: RegCodeModel[] = []
  const gen = (len:number)=>{
    const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'
    if (typeof crypto !== 'undefined' && 'getRandomValues' in crypto) {
      const array = new Uint8Array(len); crypto.getRandomValues(array)
      return Array.from(array, n=>alphabet[n%alphabet.length]).join('')
    } else {
      let s=''; for(let i=0;i<len;i++){ s+=alphabet[Math.floor(Math.random()*alphabet.length)] } return s
    }
  }
  for (let i = 0; i < payload.quantity; i++) {
    const code = gen(20)
    const body: any = {
      code,
      app_id: payload.app_id,
      status: 0,
      valid_days: payload.code_type === 0 ? (payload.valid_days ?? 0) : 0,
      max_devices: 1,
      code_type: payload.code_type,
      expire_time: null,
      total_count: payload.code_type === 1 ? (payload.total_count ?? 1) : null,
    }
    const resp = await request.post('/admin/reg_codes', body) as ApiResponse<RegCodeModel>
    items.push(resp.data)
  }
  return items
}

export const bindRegCode = async (payload: BindRegCodeReq) => {
  const response = await request.post('/reg/bind', payload) as ApiResponse<RegCodeBindCheckResp>
  return response.data
}

export const checkRegDevice = async (payload: CheckRegDeviceReq) => {
  const response = await request.post('/reg/check', payload) as ApiResponse<RegCodeBindCheckResp>
  return response.data
}

export const useRegCount = async (payload: UseCountReq) => {
  const response = await request.post('/reg/usecount', payload) as ApiResponse<UseCountResp>
  return response.data
}


