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
  <div class="reader-shell">
    <div class="legal-layout">
      <article class="reader-article">
        <div class="reader-eyebrow">legal</div>
        <h1>Privacy Policy</h1>
        <p class="reader-dek">Last updated: February 2026</p>

        <div class="reader-prose mt-11">
          <div class="lesson-content" data-lesson-content>
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
              We use <strong>Google Analytics</strong> to understand which pages
              people visit, how they found us, and what devices they're using.
              This data is aggregated and anonymous. I can't see <em>you</em>
              specifically&mdash;I can see that 40 people read the memory
              management chapter last Tuesday. That helps me know what to write
              next.
            </p>
            <p>
              If you have concerns about this, please reach out. I'm open to
              building a custom analytics solution if there's enough reason to.
            </p>
            <p>
              Google Analytics uses cookies to do this. You can opt out by
              installing the
              <a to="https://tools.google.com/dlpage/gaoptout" target="_blank" rel="noopener">Google
                Analytics Opt-out Add-on</a>, or by adjusting your browser's
              cookie settings.
            </p>

            <h2>Cookies</h2>
            <p>
              We use cookies for two things: keeping you logged in (session
              cookies) and analytics (Google Analytics cookies). That's the full
              list. No tracking pixels following you around the internet, no
              advertising cookies, no third-party ad networks. If you block
              cookies, the site will still work&mdash;you'll just need to log in
              each time.
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
                  <td>Google Analytics</td>
                  <td>Anonymous usage data</td>
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

            <hr class="legal-hr" >
            <p>
              The short version of all of this: I don't collect your data
              myself&mdash;services like Google Analytics handle usage tracking,
              and I only view aggregated info on their dashboards. Nothing is
              sold. If you have questions, I'm reachable.
            </p>
            <p>&mdash; Aryan</p>
          </div>
        </div>
      </article>
    </div>
  </div>
</template>

<style scoped>
.legal-layout {
  max-width: 760px;
  margin: 0 auto;
  padding: 72px 40px 96px;
}

.legal-hr {
  border: none;
  height: 1px;
  background: var(--color-read-line-soft);
  margin: 40px 0 28px;
}

@media (max-width: 880px) {
  .legal-layout {
    padding: 40px 20px 80px;
  }
}
</style>
