import { test, expect } from '@playwright/test'

test.describe('About page', () => {
  test('/about loads', async ({ page }) => {
    await page.goto('/about')
    await expect(page.locator('h1').first()).toBeVisible()
  })

  test('about page has an h1', async ({ page }) => {
    await page.goto('/about')
    const h1 = page.locator('h1').first()
    await expect(h1).toBeVisible()
    await expect(h1).toContainText('About')
  })
})
