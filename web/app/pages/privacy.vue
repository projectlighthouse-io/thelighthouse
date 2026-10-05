<script setup lang="ts">
import '@/assets/css/reader.css'

useSeo({
  title: 'Privacy Policy — projectlighthouse',
  description: 'projectlighthouse privacy policy. What data we collect, why, and how it is handled.',
})

useJsonLd('page', {
  '@type': 'WebPage',
  'name': 'Privacy Policy',
  'publisher': { '@type': 'Organization', 'name': SITE.name, 'url': SITE.url },
})
</script>

<template>
  <article class="page lh-text lh-gap">
    <header class="lh-head">
      <p class="lh-eyebrow">legal</p>
      <h1 class="lh-h1">Privacy Policy</h1>
      <p class="lh-sub">Last updated: October 2026</p>
    </header>

    <div class="lesson-content body is-legal">
            <p>Hi, it's Aryan again.</p>

            <p>
              Privacy policies are usually walls of legal text that nobody reads. I
              don't want to do that. You're here to learn programming, not to decode
              a contract. So let me just tell you, plainly, what happens with your
              data when you use projectlighthouse.
            </p>

            <h2>What's stored, and why</h2>
            <p>
              When you sign up, you authenticate through <strong>Google</strong> or
              <strong>GitHub</strong> via OAuth. There are no passwords
              involved&mdash;authentication is handled entirely by your provider.
              What we receive is your name, email address, and profile photo.
              That's it. We use this to create your account and show your name on
              your public profile.
            </p>
            <p>
              If you subscribe to a paid plan, payments are handled entirely by
              <strong>Stripe</strong>. Your card number never touches our servers.
              Stripe shares with us only what we need: a customer ID, your
              subscription status, and invoice records so we can show you your
              billing history.
            </p>
            <p>
              If you sign up for the newsletter, your email goes to
              <strong>Kit</strong> (ConvertKit), which handles delivery. Every
              email has an unsubscribe link. One click and you're out. No hard
              feelings.
            </p>

            <h2>Analytics</h2>
            <p>
              We use <strong>Cloudflare Web Analytics</strong> to understand
              which pages people visit, where they came from, and what devices
              they're using. It sets no cookies, doesn't fingerprint you, and
              doesn't follow you to other sites. I can't see <em>you</em>
              specifically&mdash;I can see that 40 people read the memory
              management chapter last Tuesday. That helps me know what to write
              next.
            </p>
            <p>
              The site sits behind Cloudflare anyway, so this adds no new party
              handling your visits. You can read how it works in
              <a href="https://www.cloudflare.com/web-analytics/" target="_blank" rel="noopener">Cloudflare's
                own description</a>. If you block its script, nothing on the site
              breaks.
            </p>

            <h2>Cookies</h2>
            <p>
              We use cookies for one thing: keeping you signed in. That's the
              full list. No analytics cookies, no tracking pixels following you
              around the internet, no advertising cookies, no third-party ad
              networks. If you block cookies, the site still works&mdash;you
              just can't stay signed in.
            </p>

            <h2>Your notes and progress</h2>
            <p>
              Any notes you take on lessons, your completion progress, and your
              lab work are stored in our database. This data belongs to you. If
              you want it exported or deleted, just reach out and I'll take care
              of it.
            </p>

            <h2>Where your data lives</h2>
            <p>
              The application runs on infrastructure provided by
              <strong>DigitalOcean</strong>. Your data is stored on servers in the
              United Kingdom. We use standard security practices&mdash;encrypted
              connections (TLS), secure token storage, and access
              controls&mdash;to keep things safe. If you're building systems, you
              probably know what those mean. If not, we have a book for that.
            </p>

            <h2>Third-party services, at a glance</h2>
            <p>Here's every external service that touches your data, and what they see:</p>
            <table>
              <thead>
                <tr>
                  <th>Service</th>
                  <th>What they receive</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td>Google / GitHub</td>
                  <td>OAuth tokens (for login)</td>
                </tr>
                <tr>
                  <td>Stripe</td>
                  <td>Payment and billing info</td>
                </tr>
                <tr>
                  <td>Cloudflare</td>
                  <td>Every request, as the network in front of the site, and anonymous, cookieless page views (Web Analytics)</td>
                </tr>
                <tr>
                  <td>Kit (ConvertKit)</td>
                  <td>Your email (if you subscribe to the newsletter)</td>
                </tr>
              </tbody>
            </table>
            <p>
              Each of these services has their own privacy policy. I chose them
              because they're reputable, widely used, and I trust them with my own
              data too.
            </p>

            <h2>Your rights</h2>
            <p>
              You can ask to see what data we have about you. You can ask us to
              delete it. You can ask us to export it. You can unsubscribe from
              emails. You can close your account. None of this requires a lawyer
              or a formal request&mdash;just send me a message.
            </p>

            <h2>Changes to this policy</h2>
            <p>
              If anything here changes, I'll update this page. If it's something
              significant, I'll mention it in the newsletter. I'm not going to
              quietly change the rules on you.
            </p>

            <hr>
            <p>
              The short version of all of this: I don't collect your data
              myself&mdash;Cloudflare's cookieless analytics handle usage
              counts, and I only view aggregated numbers on its dashboard. Nothing is
              sold. If you have questions, I'm reachable.
            </p>
            <p>&mdash; Aryan</p>
    </div>
  </article>
</template>

<style scoped>
.body { margin-top: var(--space-12); }
</style>
