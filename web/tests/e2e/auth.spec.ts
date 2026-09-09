import { test, expect } from '@playwright/test'

const PROVIDER_EMAIL = 'provider@example.com'
const PATIENT_EMAIL = 'patient@example.com'
const PASSWORD = 'Password123!'

// Both browser projects run against one seeded database, so test-created
// records need a per-run unique suffix (slugs and emails are unique columns).
const runId = `${Date.now()}-${Math.floor(Math.random() * 1e6)}`

async function loginWithPassword(
  page: import('@playwright/test').Page,
  email: string,
  password: string
) {
  await page.goto('/login')
  await page.locator('#login-email').fill(email)
  await page.locator('#login-password').fill(password)
  await page.getByRole('button', { name: 'Sign in', exact: true }).click()
}

test.describe('Authentication and account areas', () => {
  test('unauthenticated users are redirected from the dashboard to login', async ({ page }) => {
    await page.goto('/dashboard')
    await expect(page).toHaveURL(/\/login\?redirect=/)
  })

  test('register a provider account and land on the dashboard', async ({ page }) => {
    const email = `e2e-provider-${runId}@example.com`

    await page.goto('/register')
    await page.locator('#register-email').fill(email)
    await page.locator('#register-password').fill(PASSWORD)
    await page.locator('#register-confirm').fill(PASSWORD)
    await page.getByRole('button', { name: 'Create provider account' }).click()

    await expect(page).toHaveURL(/\/dashboard$/)
    await expect(page.locator('h1')).toContainText('Provider dashboard')
    await expect(page.getByText('No clinics yet.')).toBeVisible()
  })

  test('provider logs in, sees the seeded clinic, and creates a clinic via the UI', async ({
    page,
  }) => {
    await loginWithPassword(page, PROVIDER_EMAIL, PASSWORD)

    await expect(page).toHaveURL(/\/dashboard$/)
    await expect(page.locator('h1')).toContainText('Provider dashboard')
    await expect(page.getByTestId('clinic-card').filter({ hasText: 'Istanbul Smile Clinic' }))
      .toBeVisible()

    const slug = `e2e-ui-clinic-${runId}`
    await page.getByRole('link', { name: 'Add clinic' }).click()
    await expect(page).toHaveURL(/\/dashboard\/clinics\/new$/)
    const clinicName = `E2E UI Clinic ${runId}`
    await page.locator('#clinic-name').fill(clinicName)
    await page.locator('#clinic-slug').fill(slug)
    await page.locator('#clinic-country').fill('TR')
    await page.locator('#clinic-city').fill('Antalya')
    await page.locator('#clinic-accreditations').fill('JCI')
    await page.getByRole('button', { name: 'Create clinic' }).click()

    await expect(page).toHaveURL(/\/dashboard$/)
    await expect(page.getByTestId('clinic-card').filter({ hasText: clinicName })).toBeVisible()
  })

  test('session-aware header shows the account menu and logout works', async ({ page }) => {
    await loginWithPassword(page, PROVIDER_EMAIL, PASSWORD)
    await expect(page).toHaveURL(/\/dashboard$/)

    await page.getByRole('button', { name: 'Account menu' }).click()
    await expect(page.getByRole('menuitem', { name: 'Provider dashboard' })).toBeVisible()
    await page.getByRole('menuitem', { name: 'Sign out' }).click()

    await expect(page).toHaveURL(/\/$/)
    await expect(page.getByRole('link', { name: 'Sign in' }).first()).toBeVisible()
  })

  test('magic-link request shows the check-your-email state', async ({ page }) => {
    await page.goto('/login')
    await page.getByRole('tab', { name: 'Email link' }).click()
    await page.locator('#magic-email').fill(`e2e-magic-${runId}@example.com`)
    await page.getByRole('button', { name: 'Email me a sign-in link' }).click()

    await expect(page.getByTestId('check-your-email')).toBeVisible()
    await expect(page.getByText('Check your email')).toBeVisible()
  })

  test('patient submits an inquiry and sees it in tracking', async ({ page }) => {
    await loginWithPassword(page, PATIENT_EMAIL, PASSWORD)
    await expect(page).toHaveURL(/\/account$/)

    await page.goto('/quote?clinic=istanbul-smile-clinic')
    await expect(page.locator('#quote-email')).toHaveValue(PATIENT_EMAIL)
    await page.locator('#quote-dates').fill('October 2026')
    await page.locator('#quote-notes').fill(`E2E inquiry ${runId}`)
    await page.getByRole('button', { name: 'Request quote' }).click()
    await expect(page.getByRole('heading', { name: 'Thank you!' })).toBeVisible()

    await page.goto('/account')
    const card = page.getByTestId('inquiry-card').filter({ hasText: 'Istanbul Smile Clinic' })
    await expect(card.first()).toBeVisible()
    await expect(card.first().getByText('New', { exact: true })).toBeVisible()
  })

  test('provider inbox lists the patient inquiry and can update its status', async ({ page }) => {
    // Submit an inquiry as the patient first so the provider inbox has data.
    await loginWithPassword(page, PATIENT_EMAIL, PASSWORD)
    await expect(page).toHaveURL(/\/account$/)
    await page.goto('/quote?clinic=istanbul-smile-clinic')
    const note = `Inbox visibility check ${runId}`
    await page.locator('#quote-notes').fill(note)
    await page.getByRole('button', { name: 'Request quote' }).click()
    await expect(page.getByRole('heading', { name: 'Thank you!' })).toBeVisible()

    await page.getByRole('button', { name: 'Account menu' }).click()
    await page.getByRole('menuitem', { name: 'Sign out' }).click()

    await loginWithPassword(page, PROVIDER_EMAIL, PASSWORD)
    await expect(page).toHaveURL(/\/dashboard$/)

    const card = page
      .getByTestId('inquiry-card')
      .filter({ hasText: note })
      .first()
    await expect(card).toBeVisible()
    await card.locator('select').selectOption('contacted')
    await expect(card.getByText('Contacted', { exact: true })).toBeVisible()
  })

  test('patients cannot open the provider dashboard', async ({ page }) => {
    await loginWithPassword(page, PATIENT_EMAIL, PASSWORD)
    await expect(page).toHaveURL(/\/account$/)

    await page.goto('/dashboard')
    await expect(page).toHaveURL(/\/$/)
  })
})
