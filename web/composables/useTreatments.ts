export interface Treatment {
  id: string
  slug: string
  name: string
  category: string
  description: string | null
  created_at: string
}

export interface TreatmentsResponse {
  treatments: Treatment[]
}

export const useTreatments = () => {
  const config = useRuntimeConfig()
  const baseURL = config.public.apiUrl

  const getTreatments = async () => {
    return $fetch<TreatmentsResponse>(`${baseURL}/treatments`)
  }

  return { getTreatments }
}
