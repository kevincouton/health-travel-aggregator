import { test, expect } from '@playwright/test'

test.describe('Theme toggle', () => {
  test('theme toggle button is visible', async ({ page }) => {
    await page.goto('/')
    const themeButton = page.locator('button[aria-label*="theme"], button[aria-label*="Switch to"]').first()
    await expect(themeButton).toBeVisible()
  })

  test('clicking theme toggle changes the theme', async ({ page }) => {
    await page.goto('/')
    const themeButton = page.locator('button[aria-label*="theme"], button[aria-label*="Switch to"]').first()

    const html = page.locator('html')
    const initialHasDark = await html.evaluate((el) => el.classList.contains('dark'))

    await themeButton.click()

    const afterHasDark = await html.evaluate((el) => el.classList.contains('dark'))
    expect(afterHasDark).not.toBe(initialHasDark)
  })

  test('html element gets dark class when toggled to dark', async ({ page }) => {
    await page.goto('/')
    const html = page.locator('html')
    const themeButton = page.locator('button[aria-label*="theme"], button[aria-label*="Switch to"]').first()

    const isDark = await html.evaluate((el) => el.classList.contains('dark'))
    if (!isDark) {
      await themeButton.click()
    }

    await expect(html).toHaveClass(/dark/)
  })
})
