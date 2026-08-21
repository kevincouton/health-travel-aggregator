export interface ClinicFilters {
  country_code?: string
  city?: string
  q?: string
}

export interface Clinic {
  id: string
  owner_user_id: string
  name: string
  slug: string
  country_code: string
  city: string
  accreditations: string[]
  description: string | null
  status: string
  created_at: string
}

export const useClinics = () => {
  const config = useRuntimeConfig()
  const baseURL = config.public.apiUrl

  const buildQuery = (filters: ClinicFilters = {}): string => {
    const params = new URLSearchParams()
    if (filters.country_code) params.set('country_code', filters.country_code)
    if (filters.city) params.set('city', filters.city)
    const query = params.toString()
    return query ? `?${query}` : ''
  }

  const getClinics = async (filters: ClinicFilters = {}) => {
    return $fetch<Clinic[]>(`${baseURL}/clinics${buildQuery(filters)}`)
  }

  const getClinic = async (slug: string) => {
    return $fetch<Clinic>(`${baseURL}/clinics/${encodeURIComponent(slug)}`)
  }

  return { getClinics, getClinic }
}
