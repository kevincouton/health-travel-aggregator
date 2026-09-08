# Clinic Data Sources — Legal & Feasibility Assessment

**Purpose:** this document is the owner's legal sign-off artifact for Phase 1
clinic-data acquisition. Every source the collectors use — and every source
evaluated but rejected — is listed with its terms-of-service (ToS) posture,
robots.txt status, and scraping feasibility. Nothing here is a legal
conclusion; items flagged **NEEDS OWNER LEGAL REVIEW** must be signed off
before the corresponding data is shown publicly in production.

**Collector etiquette (all sources):** one HTTP fetch of one listing page per
collector per run (no recursive crawling, no detail-page fetching), a
descriptive User-Agent
(`health-travel-aggregator-collectors/0.1 (+https://health-travel.lucanian.app; contact: admin@lucanian.app)`),
and generated (not copied) description text so no source prose is republished.
Only factual data is stored: organization name, country/city, accreditation or
membership label, and the source URL for attribution.

---

## Sources in use

### 1. MHTC — Malaysia Healthcare Travel Council member hospitals (`mhtc`)

- **URL:** https://www.malaysiahealthcare.org/find-hospital
- **Data:** 91 member hospitals/centres with Malaysian state, membership tier
  (MHTC Member / MHTC Elite Partner), and website URL.
- **ToS/licensing:** no site-wide ToS or data license could be located on the
  site. MHTC is a Malaysian Ministry of Health initiative; the directory is
  published for public reference. No explicit reuse license is granted.
  **NEEDS OWNER LEGAL REVIEW** — republishing the member list (even as
  factual data) has no explicit license basis; the risk is low (facts are not
  copyrightable in most jurisdictions, and accreditation/membership status is
  a matter of public record) but is not zero (EU-style database rights do not
  apply to a Malaysian government directory, yet Malaysian law was not
  reviewed).
- **robots.txt:** https://www.malaysiahealthcare.org/robots.txt — permits
  crawling; only paginated `/news`, `/press-release`, `/events` URLs are
  disallowed. `/find-hospital` is allowed.
- **Feasibility:** fully server-rendered (Webflow CMS collection), single
  page, stable markup (`div.hospital-item`, `h5.hospital-tile`). Parser:
  `collectors/src/sources/mhtc.rs`; fixture:
  `collectors/tests/fixtures/mhtc_find_hospital.html` (captured 2026-09-08).
- **Caveats:** state-level location granularity only; the state capital is
  stored as the city and records land in `pending` moderation.

### 2. Temos accredited & assessed partners (`temos`)

- **URL:** https://temos-accreditation.com/AccreditedPartners/List.aspx
- **Data:** 83 accredited/assessed healthcare organizations worldwide with
  country, city, care level (secondary/tertiary/dental/etc.), and program
  tier (Temos Accredited / Temos Excellence).
- **ToS/licensing:** no terms of use or data license found on
  temos-accreditation.com or the linked https://www.temos-worldwide.com.
  **NEEDS OWNER LEGAL REVIEW** — Temos is a private German accreditation
  company; its partner list is factual (who holds which accreditation) but
  the compilation is their commercial product. Displaying "Temos Accredited"
  badges may also implicate trademark/certification-mark usage rules.
- **robots.txt:** https://temos-accreditation.com/robots.txt returns **404**
  (no robots.txt exists), so no crawl restrictions are stated. Absence of
  robots.txt is not permission; it is absence of a stated policy.
- **Feasibility:** fully server-rendered ASP.NET table
  (`table#AccreditedPartnersList tr.hos`), single page, stable per-partner
  numeric IDs used as `external_ref`. Parser: `collectors/src/sources/temos.rs`;
  fixture: `collectors/tests/fixtures/temos_accredited_partners.html`
  (captured 2026-09-08).
- **Caveats:** some rows have messy city fields (district prefixes, trailing
  punctuation); normalized best-effort. "Northern Cyprus" is mapped to `CY`.

### 3. GHA — Global Healthcare Accreditation directory (`gha`)

- **URL:** https://www.globalhealthcareaccreditation.com/accredited-and-certified-organizations
- **Data:** ~28 accredited healthcare organizations (hospitals, ambulatory
  centers) with country, city, and accreditation program. Facilitator,
  corporate-entity, and hotel cards are deliberately excluded.
- **ToS/licensing:** no ToS or data license found on the site.
  **NEEDS OWNER LEGAL REVIEW** — same posture as Temos: private US
  accreditation company; factual accreditation status vs. their compiled
  directory and certification marks.
- **robots.txt:** https://www.globalhealthcareaccreditation.com/robots.txt —
  disallows only `ia_archiver`; general crawling is permitted and a sitemap
  is advertised.
- **Feasibility:** server-rendered Webflow directory cards
  (`div.directory-card`), single page. One record has country/city swapped
  upstream ("Kyiv"/"Ukraine"); the parser repairs this. Organizations
  appearing under multiple program tabs are deduplicated with merged
  accreditation labels. Parser: `collectors/src/sources/gha.rs`; fixture:
  `collectors/tests/fixtures/gha_accredited_organizations.html` (captured
  2026-09-08).

---

## Sources evaluated and rejected

### JCI — Joint Commission International accredited organizations

- **URL:** https://www.jointcommissioninternational.org/about-jci/jci-accredited-organizations/
- **Why rejected:** the listing endpoint returns **HTTP 403** to our
  documented User-Agent (bot mitigation interstitial), and the accredited-
  organization search is a JavaScript-driven application — no server-rendered
  listing to parse. robots.txt itself is permissive for generic crawlers
  (`Allow: /`, `Content-Signal: search=yes`), but the effective access
  control says otherwise. **Not forced.** Revisit via an official data
  partnership or licensed feed — JCI sells/licensing accreditation data.

### USHAŞ / HealthTürkiye (Türkiye health-tourism authorized providers)

- **URL:** https://www.ushas.gov.tr/saglik-turizmi-yetki-belgesi-olan-tesisler-ve-araci-kuruluslar
- **Why rejected:** robots.txt permits crawling, but the authorized-facility
  and authorized-agency lists are rendered **entirely client-side** (the HTML
  contains only a spinner and a Next.js app shell; all paginated variants are
  byte-identical). The underlying JSON API is undocumented/private, so
  collecting would mean reverse-engineering a private API — not defensible.
  **Not forced.** Strong future candidate if USHAŞ publishes an official open
  dataset or grants API access (the data is a Turkish government registry
  under the 2017 International Health Tourism regulation).

### NABH (India) accredited hospitals

- **URL:** https://nabh.co (accredited-HCO search at
  https://hcoaccreditation.nabh.co)
- **Why rejected:** the accredited-organization search sits **behind an
  account login**; no public server-rendered list exists. Credentials-gated
  access is out of scope for public-data collection.

### GCR — Global Clinic Rating

- **URL:** https://www.gcr.org
- **Why rejected:** GCR's rankings are its commercial product; no public
  terms permitting reuse, and the clinic listings are app-rendered. High ToS
  risk, low feasibility.

### Wikidata (WDQS SPARQL hospital query)

- **URL:** https://query.wikidata.org/sparql
- **Why rejected (for now):** data is CC0 (ideal licensing), and the WDQS
  Usage Policy explicitly permits automated queries with a descriptive
  User-Agent — but https://query.wikidata.org/robots.txt states
  `User-agent: * Disallow: /sparql`, and our policy is to respect robots.txt
  as written. **NEEDS OWNER LEGAL REVIEW** to decide whether the documented
  WDQS Usage Policy (which the robots.txt arguably exists to rate-limit, not
  prohibit) is sufficient cover. If approved, this is the richest additional
  source: thousands of hospitals with city/country coordinates under CC0.

---

## Cross-cutting notes for the reviewer

- **Database-rights exposure:** the three active sources are a Malaysian
  government initiative (MHTC) and two private accreditation firms (Temos,
  GHA). We store only factual fields and generate our own description text.
  We do not copy source prose, images, logos, or ratings.
- **Attribution:** every ingested row carries `source`, `external_ref`, and
  the source URL inside its description, so provenance is auditable and a
  source can be purged with one `DELETE ... WHERE source = ...` if a rights
  holder objects.
- **Moderation gate:** all ingested listings enter as `status = 'pending'`
  and are invisible in public search until a platform admin approves them
  through the verification queue — a second human checkpoint before any
  collected data goes live.
- **Re-ingestion:** re-runs refresh name/city/accreditations but never reset
  moderation status or overwrite provider edits to owned listings.
