import { test, expect } from '@playwright/test'

test.describe('Homepage', () => {
  test('home page loads', async ({ page }) => {
    await page.goto('/')
    await expect(page.locator('h1').first()).toBeVisible()
  })

  test('header contains the platform name', async ({ page }) => {
    await page.goto('/')
    const headerLink = page.locator('header a').first()
    await expect(headerLink).toBeVisible()
    await expect(headerLink).not.toHaveText('')
  })

  test('Entities link is visible', async ({ page }) => {
    await page.goto('/')
    const menuButton = page.locator('button[aria-label="Toggle navigation menu"]').first()
    if (await menuButton.isVisible()) {
      await menuButton.click()
      await expect(page.locator('#mobile-menu').getByRole('link', { name: 'Entities' }).first()).toBeVisible()
    } else {
      await expect(page.getByRole('link', { name: 'Entities' }).first()).toBeVisible()
    }
  })

  test('About link navigates to /about', async ({ page }) => {
    await page.goto('/')
    const menuButton = page.locator('button[aria-label="Toggle navigation menu"]').first()
    if (await menuButton.isVisible()) {
      await menuButton.click()
      await expect(page.locator('#mobile-menu').getByRole('link', { name: 'About' }).first()).toBeVisible()
      await page.locator('#mobile-menu').getByRole('link', { name: 'About' }).first().click({ force: true })
    } else {
      await page.getByRole('link', { name: 'About' }).first().click()
    }
    await expect(page).toHaveURL('/about')
    await expect(page.locator('h1').first()).toBeVisible()
  })
})
