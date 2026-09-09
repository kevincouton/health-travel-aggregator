# Activation: taking SMTP email and Stripe billing live

This checklist is for the **owner**. The codebase ships with mock connectors;
both integrations activate purely through environment variables. Nothing here
touches production deploy — deploying the configured service is a separate,
later step.

Current state: **needs owner keys**. Until the variables below are set, the
server runs the mock email sender and mock payment provider (dev/CI behavior,
fully tested).

## 1. Stripe (subscriptions: Pro $99/mo, Enterprise $299/mo)

1. **Create a Stripe account** at https://dashboard.stripe.com/register and
   complete business verification. Work in **test mode** first, then repeat in
   live mode.
2. **Create products and prices** (Dashboard → Products):
   - Product "Pro" → recurring price **$99.00 USD / month**. Copy the price ID
     (`price_...`) → `STRIPE_PRICE_PRO`.
   - Product "Enterprise" → recurring price **$299.00 USD / month**. Copy the
     price ID → `STRIPE_PRICE_ENTERPRISE`.
   - The Basic plan is free ($0) and needs no Stripe object.
3. **Get the secret API key** (Dashboard → Developers → API keys) →
   `STRIPE_SECRET_KEY` (`sk_test_...` / `sk_live_...`). Setting this variable
   switches the payment provider from mock to Stripe.
4. **Create the webhook endpoint** (Dashboard → Developers → Webhooks →
   Add endpoint):
   - URL: `https://<your-api-host>/webhooks/stripe`
   - Events: `checkout.session.completed`, `customer.subscription.updated`,
     `customer.subscription.deleted`
   - Copy the endpoint's **Signing secret** (`whsec_...`) →
     `STRIPE_WEBHOOK_SECRET`.
5. **Set the env vars** (root `.env` for docker compose, or the service
   environment) and restart the API. On startup the log line from connector
   selection confirms Stripe mode; without `STRIPE_SECRET_KEY` the mock stays
   active.
6. **Verify end-to-end (test mode)**: register a provider, call
   `POST /me/subscription/checkout` with `{"plan_slug": "pro"}`, open the
   returned URL, pay with Stripe's test card `4242 4242 4242 4242`. The
   webhook then activates the Pro plan (`GET /me/subscription` shows `pro`,
   status `active`), and plan limits rise to 5 clinics / 20 packages.
   Dashboard → Developers → Webhooks shows delivery status; failed deliveries
   are retried by Stripe automatically.

Security notes: the webhook route is unauthenticated by design — the
`Stripe-Signature` HMAC (v1, 5-minute timestamp tolerance) is the
authentication. Never log or commit the keys; rotate them if exposed.

## 2. SMTP (magic-link sign-in and inquiry notification emails)

1. **Pick a mailbox/provider.** Any SMTP relay works; common options:
   - Transactional providers: Resend, Postmark, Amazon SES, Mailgun,
     Brevo — create an SMTP credential in their dashboard.
   - A mailbox on your own domain via your host (e.g. Google Workspace /
     Microsoft 365 SMTP with an app password).
   - Local/dev sanity check: a mail catcher like Mailpit with `SMTP_TLS=off`.
2. **Verify the sending domain** (SPF/DKIM) with the provider so mail isn't
   spam-foldered; use a matching From address.
3. **Set the env vars**:
   - `SMTP_HOST` (setting this switches from mock to real sending)
   - `SMTP_PORT` — optional; defaults to 587 for `starttls`, 465 for `tls`
   - `SMTP_USERNAME` / `SMTP_PASSWORD` — optional for open local relays
   - `SMTP_FROM` — e.g. `Health Travel <no-reply@yourdomain.com>` (required
     when `SMTP_HOST` is set)
   - `SMTP_TLS` — `starttls` (default), `tls` (implicit TLS, port 465), or
     `off` (local relay only)
4. **Restart the API and verify**: request a magic link
   (`POST /auth/magic-link`) and confirm the email arrives with a working
   `APP_URL/auth/verify?token=...` link. If the SMTP config is invalid the
   server logs an error and falls back to the mock sender — check startup
   logs for `SMTP email sender configured`.

## 3. What is intentionally NOT done here

- No production deploy. Choosing the host, DNS, TLS termination, and running
  migrations there is a separate later step.
- Stripe Customer Portal / self-serve cancellation and plan downgrades are not
  wired; cancellation can be handled in the Stripe dashboard and will sync via
  `customer.subscription.deleted`.
- Inquiry-notification emails reuse the same SMTP sender; no separate
  configuration is needed.
