import type { PagingResponse } from '@/types/api'

export const API_MAX_PAGE_SIZE = 100

type PageParams = {
  page: number
  page_size: number
}

export async function fetchAllPages<T>(
  fetchPage: (params: PageParams) => Promise<PagingResponse<T>>,
): Promise<T[]> {
  const list: T[] = []
  let page = 1

  while (true) {
    const response = await fetchPage({ page, page_size: API_MAX_PAGE_SIZE })
    list.push(...response.list)

    if (list.length >= response.total || response.list.length < API_MAX_PAGE_SIZE) {
      return list
    }
    page += 1
  }
}
