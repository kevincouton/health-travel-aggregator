import { $fetch } from 'ofetch'

import type { Clinic } from './useClinics'

export type ClinicStatus = 'draft' | 'pending' | 'approved' | 'suspended'
export type ClaimStatus = 'pending' | 'approved' | 'rejected'

export interface ClinicClaim {
  id: string
  clinic_id: string
  user_id: string
  message: string | null
  status: ClaimStatus
  created_at: string
  resolved_at: string | null
}

/** Platform-admin moderation endpoints. */
export const useAdmin = () => {
  const config = useRuntimeConfig()
  const baseURL = config.public.apiUrl

  const listClinics = async (status: ClinicStatus = 'pending') => {
    return $fetch<Clinic[]>(`${baseURL}/admin/clinics?status=${encodeURIComponent(status)}`, {
      credentials: 'include',
    })
  }

  const updateClinicStatus = async (id: string, status: ClinicStatus) => {
    return $fetch<Clinic>(`${baseURL}/admin/clinics/${encodeURIComponent(id)}/status`, {
      method: 'PATCH',
      body: { status },
      credentials: 'include',
    })
  }

  const flagClinic = async (id: string, reason: string) => {
    return $fetch<Clinic>(`${baseURL}/admin/clinics/${encodeURIComponent(id)}/flag`, {
      method: 'POST',
      body: { reason },
      credentials: 'include',
    })
  }

  const listClaims = async (status: ClaimStatus = 'pending') => {
    return $fetch<ClinicClaim[]>(`${baseURL}/admin/claims?status=${encodeURIComponent(status)}`, {
      credentials: 'include',
    })
  }

  const resolveClaim = async (id: string, status: ClaimStatus) => {
    return $fetch<ClinicClaim>(`${baseURL}/admin/claims/${encodeURIComponent(id)}/status`, {
      method: 'PATCH',
      body: { status },
      credentials: 'include',
    })
  }

  return { listClinics, updateClinicStatus, flagClinic, listClaims, resolveClaim }
}
