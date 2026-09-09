import { $fetch } from 'ofetch'

import type { Clinic } from './useClinics'
import type { Package } from './usePackages'

export interface ClinicPayload {
  name: string
  slug: string
  country_code: string
  city: string
  accreditations: string[]
  description?: string | null
}

export interface CreatePackagePayload {
  treatment_id: string
  name: string
  price_min?: number | null
  price_max?: number | null
  duration_days?: number | null
  inclusions?: string[]
  exclusions?: string[]
}

export interface UpdatePackagePayload {
  name: string
  price_min?: number | null
  price_max?: number | null
  duration_days?: number | null
  inclusions?: string[]
  exclusions?: string[]
  is_published: boolean
}

/** Provider-admin endpoints: the signed-in provider's clinics and packages. */
export const useProvider = () => {
  const config = useRuntimeConfig()
  const baseURL = config.public.apiUrl

  const getMyClinics = async () => {
    return $fetch<Clinic[]>(`${baseURL}/me/clinics`, { credentials: 'include' })
  }

  const createClinic = async (payload: ClinicPayload) => {
    return $fetch<Clinic>(`${baseURL}/me/clinics`, {
      method: 'POST',
      body: payload,
      credentials: 'include',
    })
  }

  const updateClinic = async (id: string, payload: ClinicPayload) => {
    return $fetch<Clinic>(`${baseURL}/me/clinics/${encodeURIComponent(id)}`, {
      method: 'PATCH',
      body: payload,
      credentials: 'include',
    })
  }

  const getMyPackages = async (clinicId: string) => {
    return $fetch<Package[]>(`${baseURL}/me/clinics/${encodeURIComponent(clinicId)}/packages`, {
      credentials: 'include',
    })
  }

  const createPackage = async (clinicId: string, payload: CreatePackagePayload) => {
    return $fetch<Package>(`${baseURL}/me/clinics/${encodeURIComponent(clinicId)}/packages`, {
      method: 'POST',
      body: payload,
      credentials: 'include',
    })
  }

  const updatePackage = async (clinicId: string, id: string, payload: UpdatePackagePayload) => {
    return $fetch<Package>(
      `${baseURL}/me/clinics/${encodeURIComponent(clinicId)}/packages/${encodeURIComponent(id)}`,
      { method: 'PATCH', body: payload, credentials: 'include' }
    )
  }

  const deletePackage = async (clinicId: string, id: string) => {
    return $fetch<{ ok: boolean }>(
      `${baseURL}/me/clinics/${encodeURIComponent(clinicId)}/packages/${encodeURIComponent(id)}`,
      { method: 'DELETE', credentials: 'include' }
    )
  }

  return {
    getMyClinics,
    createClinic,
    updateClinic,
    getMyPackages,
    createPackage,
    updatePackage,
    deletePackage,
  }
}
