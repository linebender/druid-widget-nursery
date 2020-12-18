# `druid-widget-nursery`

This repo contains (or, at the time of writing, will contain) widgets that work with `druid`. The repo follows a policy of [optimistic merging], and the idea is that having a lower barrier to merging PRs hopefully leads to a nice contributor experience, which then encourages more peole to become regular collaborators for the whole `druid` family of crates.

We don't insist that all widgets always build when updating to a newer version of `druid`, and so as you'll see the CI is allowed to fail. Fixing these build failures will often be a good opportunity for a first contribution, and people will always be willing to help out with this work either here or [on zulip][xi zulip].

So, in summary, the default assumption for PRs to this repo will be to merge, but this policy includes future PRs that might change or reverse stuff in previous PRs. For more information I recommend reading [the optimistic merging article linked here and above][optimistic merging], which offers an interesting approach to managing open source projects irrespective of its use here.

# Widgets

 - none yet. be the first!

# Widget Requests

 - A really good, general widget for laying out collections of items.
   - There are different approaches we could use (`flexbox` or `grid` from the HTML world).
   - This might be multiple widgets serving different use cases in the end.

# Links to widget crates

Maybe you have made your own collection of widgets that aren't general enough to go into `druid` proper, but will still be useful to other `druid` users.

 - [`druid-graphs`]: An alpha quality library for drawing graphs as widgets, taking inspiration from [`matplotlib`]. Currently only supports a few graph types, collaboration welcome! Works well with [`druid-lens-compose`].

[optimistic merging]: http://hintjens.com/blog:106
[xi zulip]: https://xi.zulipchat.com/
[`druid-graphs`]: https://github.com/derekdreery/druid-graphs
[`matplotlib`]: https://matplotlib.org/
[`druid-lens-compose`]: https://github.com/derekdreery/druid-lens-compose
