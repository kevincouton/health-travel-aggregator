import { test, expect } from '@playwright/test'

test.describe('Smoke', () => {
  test('homepage loads with a heading', async ({ page }) => {
    await page.goto('/')
    await expect(page.locator('h1').first()).toBeVisible()
    await expect(page.locator('h1').first()).toContainText(/medical travel/i)
  })

  test('main navigation links are present', async ({ page }) => {
    await page.goto('/')
    await expect(page.getByRole('link', { name: 'Home' }).first()).toBeVisible()
    await expect(page.getByRole('link', { name: 'Treatments' }).first()).toBeVisible()
    await expect(page.getByRole('link', { name: 'Clinics' }).first()).toBeVisible()
    await expect(page.getByRole('link', { name: 'About' }).first()).toBeVisible()
  })
})
