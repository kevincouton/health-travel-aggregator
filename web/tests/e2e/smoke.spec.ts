import { test, expect } from '@playwright/test'

test.describe('Smoke', () => {
  test('homepage loads with a heading', async ({ page }) => {
    await page.goto('/')
    await expect(page.locator('h1').first()).toBeVisible()
    await expect(page.locator('h1').first()).toContainText(/medical travel/i)
  })

  test('homepage has exactly one h1', async ({ page }) => {
    await page.goto('/')
    await expect(page.locator('h1')).toHaveCount(1)
  })

  test('mobile menu button has an accessible label', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 })
    await page.goto('/')
    const menuButton = page.locator('button[aria-controls="mobile-menu"]').first()
    await expect(menuButton).toBeVisible()
    await expect(menuButton).toHaveAttribute('aria-label', 'Toggle navigation menu')
  })

  test('main navigation links are present', async ({ page }) => {
    await page.goto('/')
    await expect(page.getByRole('link', { name: 'Home' }).first()).toBeVisible()
    await expect(page.getByRole('link', { name: 'Treatments' }).first()).toBeVisible()
    await expect(page.getByRole('link', { name: 'Clinics' }).first()).toBeVisible()
    await expect(page.getByRole('link', { name: 'About' }).first()).toBeVisible()
  })

  test('package detail page handles unknown package gracefully', async ({ page }) => {
    await page.goto('/packages/00000000-0000-0000-0000-000000000000')
    await expect(page.locator('text=Package not found.').first()).toBeVisible()
  })

  test('sitemap.xml is served with static routes', async ({ page }) => {
    const response = await page.goto('/sitemap.xml')
    expect(response).not.toBeNull()
    expect(response?.headers()['content-type']).toContain('application/xml')

    const body = await response.text()
    expect(body).toContain('<urlset')
    expect(body).toContain('/treatments')
    expect(body).toContain('/clinics')
    expect(body).toContain('/about')
  })
})
