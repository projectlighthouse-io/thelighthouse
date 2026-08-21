<script setup lang="ts">
import '@/assets/css/reader.css'

useSeo({
  title: 'Terms of Service — projectlighthouse',
  description: 'projectlighthouse terms of service. Rules and conditions for using our programming courses, labs, and platform.',
})

useJsonLd('page', {
  '@type': 'WebPage',
  'name': 'Terms of Service',
  'publisher': { '@type': 'Organization', 'name': SITE.name, 'url': SITE.url },
})
</script>

<template>
  <div class="reader-shell">
    <div class="legal-layout">
      <article class="reader-article">
        <div class="reader-eyebrow">legal</div>
        <h1>Terms of Service</h1>
        <p class="reader-dek">Last updated: February 2026</p>

        <div class="reader-prose mt-11">
          <div class="lesson-content" data-lesson-content>
            <p>Hi, it's Aryan.</p>
            <p>
              Most terms of service read like they were written by a committee of
              lawyers trying to cover every hypothetical scenario until the heat
              death of the universe. I'd rather just be straight with you about
              how this works.
            </p>
            <p>
              projectlighthouse is an educational platform. You're here to learn
              programming, build things, and get better at your craft. These terms
              exist to make sure that works well for everyone.
            </p>

            <h2>Your account</h2>
            <p>
              You sign in through Google or GitHub. One account per person.
              You're responsible for keeping your account secure&mdash;though
              since we use OAuth, there's no password on our end to worry about.
              If you notice anything strange on your account, let me know
              immediately.
            </p>
            <p>
              You must be at least 13 years old to use projectlighthouse. If
              you're under 18, you should have a parent or guardian who's aware
              you're using the platform.
            </p>

            <h2>What you get</h2>
            <p>
              projectlighthouse offers interactive books, hands-on labs, and
              coding projects. Some content is free. Some requires a paid
              subscription. What's included in each tier is shown on the
              <a to="/subscriptionsindex">subscriptions page</a>.
            </p>
            <p>
              I'm always adding and updating content. That means things might
              change&mdash;chapters get rewritten, labs get improved, new books
              get added. The platform is a living thing, not a static product.
              Your subscription gives you access to whatever is available during
              your subscription period.
            </p>

            <h2>Payments and subscriptions</h2>
            <p>
              Paid subscriptions are billed through <strong>Stripe</strong>. When
              you subscribe, you're authorizing a recurring charge at the interval
              you chose&mdash;monthly or yearly. Prices are displayed before you
              confirm.
            </p>
            <p>
              You can cancel anytime from your account settings. When you cancel,
              your access continues until the end of your current billing period.
              No partial refunds for unused time, but no surprise charges either.
              Clean and simple.
            </p>
            <p>
              If I ever need to change pricing, existing subscribers will be
              notified in advance. I won't silently bump your bill. In the event
              that your current plan is discontinued or restructured, I reserve
              the right to migrate your subscription to the most comparable
              available plan. You'll be notified before any such change takes
              effect, and you can always cancel if the new plan doesn't work for
              you.
            </p>

            <h2>Content and intellectual property</h2>
            <p>
              All the books, lessons, lab instructions, and other educational
              content on projectlighthouse are my work (or credited where
              contributed by others). You're welcome to learn from it, take notes,
              and apply what you've learned to your own projects&mdash;that's the
              whole point.
            </p>
            <p>
              What you shouldn't do is copy, redistribute, or resell the content.
              Don't scrape the site to republish lessons elsewhere. Don't share
              your account so others can access subscriber content for free. If
              you want to reference something you learned here in a blog post or
              talk, go for it&mdash;just don't reproduce the material wholesale.
            </p>
            <p>
              The code you write in labs and projects is yours. I'm not claiming
              any ownership over your work. Build something great with it.
            </p>

            <h2>Labs and infrastructure</h2>
            <p>
              Some labs provision real virtual machines for you to work in. These
              are sandboxed environments meant for learning. A few ground rules:
            </p>
            <ul>
              <li>
                Use lab environments only for the exercises they're designed
                for. Don't use them for cryptocurrency mining, hosting services,
                or anything unrelated to learning.
              </li>
              <li>
                Don't attempt to break out of the sandbox or access other users'
                environments.
              </li>
              <li>
                Lab sessions have time limits and resource caps. This keeps
                costs manageable so I can keep offering them.
              </li>
            </ul>
            <p>
              If you find a security vulnerability in the lab infrastructure, I'd
              genuinely appreciate you telling me about it rather than exploiting
              it. That's the kind of engineering ethic this platform is trying to
              foster.
            </p>

            <h2>Acceptable use</h2>
            <p>
              This should go without saying, but for the record: don't use
              projectlighthouse for anything illegal, harmful, or disruptive.
              Don't harass other users. Don't attempt to compromise the
              platform's security. Don't automate access in ways that degrade the
              service for others.
            </p>
            <p>
              If someone is clearly acting in bad faith, I reserve the right to
              suspend or terminate their account. I don't want to do that, and in
              practice I almost never have to. But the option exists.
            </p>

            <h2>Availability</h2>
            <p>
              I do my best to keep the platform running and available. But I'm
              building this with a small team, and things break sometimes. Servers
              go down. Deploys have bugs. If you've built any system of your own,
              you know how this goes.
            </p>
            <p>
              I can't guarantee 100% uptime, and I'm not liable for any losses
              caused by the platform being temporarily unavailable. What I can
              promise is that I care about reliability and I'll fix things as
              quickly as I can.
            </p>

            <h2>Disclaimer</h2>
            <p>
              The educational content on projectlighthouse is provided "as is." I
              work hard to make it accurate and useful, but I can't guarantee
              every code example will work perfectly in your specific environment,
              or that following a tutorial will produce a particular outcome.
              Programming is like that&mdash;context matters.
            </p>
            <p>
              projectlighthouse is not a substitute for professional advice in
              areas like security, compliance, or production system design. The
              content teaches concepts and skills. How you apply them is your
              responsibility.
            </p>

            <h2>Limitation of liability</h2>
            <p>
              To the maximum extent permitted by law, projectlighthouse and its
              creator are not liable for any indirect, incidental, or
              consequential damages arising from your use of the platform. Our
              total liability for any claim is limited to the amount you've paid
              us in the 12 months preceding the claim.
            </p>
            <p>
              I know that paragraph sounds more like a lawyer wrote it. It kind of
              has to. But the spirit is simple: this is an educational platform,
              not critical infrastructure. Use what you learn wisely.
            </p>

            <h2>Changes to these terms</h2>
            <p>
              I may update these terms as the platform evolves. If something
              material changes, I'll give you notice&mdash;either through email or
              a visible notice on the site. Continuing to use the platform after
              changes means you accept the updated terms.
            </p>

            <h2>Governing law</h2>
            <p>
              These terms are governed by the laws of England and Wales. Any
              disputes will be resolved under the jurisdiction of the courts of
              England and Wales. Hopefully it never comes to that&mdash;I'd rather
              just talk things through.
            </p>

            <hr class="legal-hr" >
            <p>
              The spirit behind all of this is straightforward: use the platform
              to learn, respect the content and the community, and don't do
              anything you wouldn't want done to you. That covers about 99% of
              it.
            </p>
            <p>
              If anything here is unclear or you have questions, reach out. I'd
              rather explain something than have you wonder about it.
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
