import request from '@/utils/request'
import type { ApiResponse } from '@/types'

export interface ServerVersionInfo {
  version: string
}

export const fetchServerVersion = async () => {
  const response = await request.get('/version', { suppressErrorMessage: true } as any) as ApiResponse<ServerVersionInfo>
  return response.data
}
