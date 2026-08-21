import { test, expect } from '@playwright/test'

const API_BASE_URL = process.env.API_BASE_URL || 'http://localhost:8080'

test.describe('Smoke', () => {
  test('homepage loads and has one h1', async ({ page }) => {
    await page.goto('/')
    await expect(page.locator('h1')).toHaveCount(1)
    await expect(page.locator('h1').first()).toBeVisible()
  })

  test('treatments page lists treatments', async ({ page }) => {
    await page.goto('/treatments')
    await expect(page.locator('h1')).toContainText(/Treatments/i)
    const items = page.locator('ul > li')
    await expect(items.first()).toBeVisible()
    await expect(items).toHaveCount.greaterThan(0)
  })

  test('clinics search returns the seeded clinic', async ({ page }) => {
    // Navigate directly with the search query so the result is rendered on the
    // server and avoids cross-origin client requests to the API.
    await page.goto('/clinics?q=Istanbul')
    const clinicCard = page.locator('text=Istanbul Smile Clinic').first()
    await expect(clinicCard).toBeVisible()
  })

  test('clinic page loads', async ({ page }) => {
    await page.goto('/clinics/istanbul-smile-clinic')
    await expect(page.locator('h1')).toContainText('Istanbul Smile Clinic')
    await expect(page.locator('text=Istanbul')).toBeVisible()
  })

  test('provider can log in and create a clinic', async ({ request }) => {
    const api = await request.newContext({ baseURL: API_BASE_URL })

    try {
      const login = await api.post('/auth/provider/login', {
        data: {
          email: 'provider@example.com',
          password: 'Password123!',
        },
      })
      expect(login.ok()).toBeTruthy()

      const slug = `e2e-test-clinic-${Date.now()}`
      const create = await api.post('/me/clinics', {
        data: {
          name: 'E2E Test Clinic',
          slug,
          country_code: 'US',
          city: 'New York',
          accreditations: ['JCI'],
          description: 'Created by the Playwright smoke test.',
        },
      })
      expect(create.ok()).toBeTruthy()

      const list = await api.get('/me/clinics')
      expect(list.ok()).toBeTruthy()
      const clinics: Array<{ slug: string }> = await list.json()
      expect(clinics.some((c) => c.slug === slug)).toBe(true)
    } finally {
      await api.dispose()
    }
  })
})
