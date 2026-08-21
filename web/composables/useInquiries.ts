import { $fetch } from 'ofetch'

export interface Inquiry {
  id: string
  patient_user_id: string
  clinic_id: string
  package_id: string | null
  status: 'new' | 'contacted' | 'converted' | 'closed'
  medical_notes: string | null
  preferred_dates: string | null
  contact_email: string
  created_at: string
  updated_at: string
}

export interface CreateInquiryPayload {
  clinic_id: string
  package_id?: string | null
  medical_notes?: string
  preferred_dates?: string
  contact_email: string
}

export type InquiryStatus = Inquiry['status']

export const useInquiries = () => {
  const config = useRuntimeConfig()
  const baseURL = config.public.apiUrl

  const createInquiry = async (payload: CreateInquiryPayload) => {
    return $fetch<Inquiry>(`${baseURL}/inquiries`, {
      method: 'POST',
      body: payload,
      credentials: 'include',
    })
  }

  const getMyInquiries = async () => {
    return $fetch<Inquiry[]>(`${baseURL}/me/inquiries`, {
      credentials: 'include',
    })
  }

  const updateInquiryStatus = async (id: string, status: InquiryStatus) => {
    return $fetch<Inquiry>(`${baseURL}/me/inquiries/${encodeURIComponent(id)}/status`, {
      method: 'PATCH',
      body: { status },
      credentials: 'include',
    })
  }

  return { createInquiry, getMyInquiries, updateInquiryStatus }
}
