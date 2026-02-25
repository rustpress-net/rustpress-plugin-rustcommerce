// RustCommerce Screenshot Capture Script
// Uses Playwright to navigate the admin UI and capture 22+ screenshots

import { chromium } from 'playwright';
import { join } from 'path';

const BASE_URL = 'http://localhost:5173/admin';
const SCREENSHOT_DIR = process.argv[2] || '.';

async function delay(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function capture(page, name, path, options = {}) {
  const url = `${BASE_URL}${path}`;
  console.log(`  Capturing: ${name} -> ${url}`);
  await page.goto(url, { waitUntil: 'networkidle', timeout: 15000 }).catch(() => {});
  await delay(options.delay || 1500);

  // Dismiss any modals or overlays
  try {
    await page.keyboard.press('Escape');
    await delay(300);
  } catch {}

  const filepath = join(SCREENSHOT_DIR, name);
  await page.screenshot({
    path: filepath,
    fullPage: options.fullPage || false,
    type: 'png'
  });
  console.log(`  Saved: ${filepath}`);
}

async function main() {
  console.log('Launching browser...');
  const browser = await chromium.launch({
    headless: true,
    args: ['--no-sandbox', '--disable-gpu']
  });

  const context = await browser.newContext({
    viewport: { width: 1440, height: 900 },
    colorScheme: 'light',
    locale: 'en-US'
  });

  // Set a fake auth token so the app doesn't redirect to login
  await context.addCookies([{
    name: 'auth_token',
    value: 'demo-admin-token',
    domain: 'localhost',
    path: '/'
  }]);

  const page = await context.newPage();

  // Set localStorage auth token
  await page.goto(BASE_URL, { waitUntil: 'networkidle', timeout: 15000 }).catch(() => {});
  await page.evaluate(() => {
    localStorage.setItem('auth_token', 'demo-admin-token');
    localStorage.setItem('rustpress-user', JSON.stringify({
      id: '1', username: 'admin', email: 'admin@rustpress.net',
      display_name: 'Admin', role: 'administrator'
    }));
  });
  await delay(1000);

  console.log('\nCapturing screenshots...\n');

  // 1. Admin Login page
  await capture(page, '01_admin_login.png', '/login', { delay: 2000 });

  // Navigate back with auth
  await page.goto(BASE_URL, { waitUntil: 'networkidle', timeout: 15000 }).catch(() => {});
  await delay(1000);

  // 2. RustCommerce Dashboard
  await capture(page, '02_store_dashboard.png', '/store/dashboard', { delay: 2000 });

  // 3. Dashboard metrics close-up (same page, already loaded)
  await capture(page, '03_dashboard_metrics.png', '/store/dashboard', { delay: 500 });

  // 4. Create product page
  await capture(page, '04_create_product.png', '/store/products/new', { delay: 2000 });

  // 5. Product variants tab
  await page.goto(`${BASE_URL}/store/products/new`, { waitUntil: 'networkidle', timeout: 15000 }).catch(() => {});
  await delay(1500);
  try {
    // Click the Variants tab if it exists
    const variantsTab = await page.$('text=Variants');
    if (variantsTab) await variantsTab.click();
    await delay(800);
  } catch {}
  await page.screenshot({ path: join(SCREENSHOT_DIR, '05_product_variants.png'), type: 'png' });
  console.log('  Saved: 05_product_variants.png');

  // 6. Product images tab
  try {
    const imagesTab = await page.$('text=Images');
    if (imagesTab) await imagesTab.click();
    await delay(800);
  } catch {}
  await page.screenshot({ path: join(SCREENSHOT_DIR, '06_product_images.png'), type: 'png' });
  console.log('  Saved: 06_product_images.png');

  // 7. Edit product (same editor, different context)
  await capture(page, '07_edit_product.png', '/store/products/new', { delay: 1500 });

  // 8. Delete product (capture from product list with action menu)
  await capture(page, '08_delete_product.png', '/store/products', { delay: 2000 });

  // 9. Product list with filters
  await capture(page, '09_product_list_filters.png', '/store/products', { delay: 1500 });

  // 10. Bulk actions (same product list)
  await capture(page, '10_bulk_actions.png', '/store/products', { delay: 500 });

  // 11. Categories (via product editor sidebar or settings)
  await capture(page, '11_categories.png', '/store/products/new', { delay: 1500 });

  // 12. Create order
  await capture(page, '12_create_order.png', '/store/orders', { delay: 2000 });

  // 13. Order list with status filters
  await capture(page, '13_order_list.png', '/store/orders', { delay: 1500 });

  // 14. Order detail
  await capture(page, '14_order_detail.png', '/store/orders/demo-order-1', { delay: 2000 });

  // 15. Update order status (same order detail page)
  await capture(page, '15_update_order_status.png', '/store/orders/demo-order-1', { delay: 500 });

  // 16. Customer list
  await capture(page, '16_customer_list.png', '/store/customers', { delay: 2000 });

  // 17. Customer detail
  await capture(page, '17_customer_detail.png', '/store/customers/demo-customer-1', { delay: 2000 });

  // 18. General settings
  await capture(page, '18_general_settings.png', '/store/settings/general', { delay: 2000 });

  // 19. Payment settings
  await capture(page, '19_payment_settings.png', '/store/settings/payments', { delay: 2000 });

  // 20. Shipping settings
  await capture(page, '20_shipping_settings.png', '/store/settings/shipping', { delay: 2000 });

  // 21. Tax settings
  await capture(page, '21_tax_settings.png', '/store/settings/tax', { delay: 2000 });

  // 22. Search/filter products
  await capture(page, '22_search_products.png', '/store/products', { delay: 1500 });

  // Bonus: Coupons page
  await capture(page, '23_coupon_manager.png', '/store/coupons', { delay: 2000 });

  // Bonus: Review moderation
  await capture(page, '24_review_moderation.png', '/store/reviews', { delay: 2000 });

  // Bonus: Email settings
  await capture(page, '25_email_settings.png', '/store/settings/email', { delay: 2000 });

  // Bonus: Main enterprise dashboard
  await capture(page, '26_enterprise_dashboard.png', '/dashboard', { delay: 2000 });

  // Bonus: Plugin store showing RustCommerce
  await capture(page, '27_plugin_store.png', '/plugins', { delay: 2000 });

  await browser.close();
  console.log('\nDone! All screenshots captured.');
}

main().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});
