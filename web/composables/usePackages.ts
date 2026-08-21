export interface Package {
  id: string
  clinic_id: string
  treatment_id: string
  name: string
  price_min: number | null
  price_max: number | null
  duration_days: number | null
  inclusions: string[]
  exclusions: string[]
  is_published: boolean
  created_at: string
  updated_at: string
  clinic_name?: string
  clinic_slug?: string
  treatment_name?: string
}

export const usePackages = () => {
  const config = useRuntimeConfig()
  const baseURL = config.public.apiUrl

  const getPackage = async (id: string) => {
    return $fetch<Package>(`${baseURL}/packages/${encodeURIComponent(id)}`)
  }

  const getClinicPackages = async (slug: string) => {
    return $fetch<Package[]>(`${baseURL}/clinics/${encodeURIComponent(slug)}/packages`)
  }

  return { getPackage, getClinicPackages }
}
