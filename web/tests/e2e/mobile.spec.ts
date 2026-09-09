import { test, expect } from '@playwright/test'

test.describe('Mobile navigation', () => {
  test('mobile nav hamburger button is visible', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 })
    await page.goto('/')
    const menuButton = page.locator('button[aria-label="Toggle navigation menu"]').first()
    await expect(menuButton).toBeVisible()
  })

  test('clicking hamburger opens the menu', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 })
    await page.goto('/')
    const menuButton = page.locator('button[aria-label="Toggle navigation menu"]').first()
    await menuButton.click()
    const mobileMenu = page.locator('#mobile-menu').first()
    await expect(mobileMenu).toBeVisible()
  })

  test('nav links are accessible in the mobile menu', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 })
    await page.goto('/')
    await page.locator('button[aria-label="Toggle navigation menu"]').first().click()
    const mobileMenu = page.locator('#mobile-menu').first()
    await expect(mobileMenu.getByRole('link', { name: 'Clinics' }).first()).toBeVisible()
    await expect(mobileMenu.getByRole('link', { name: 'About' }).first()).toBeVisible()
  })
})
