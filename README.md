# `druid-widget-nursery`

This repo contains (or, at the time of writing, will contain) widgets that work with `druid`. The repo follows a policy of [optimistic merging], and the idea is that having a lower barrier to merging PRs hopefully leads to a nice contributor experience, which then encourages more people to become regular collaborators for the whole `druid` family of crates.

We don't insist that all widgets always build when updating to a newer version of `druid`, and so as you'll see the CI is allowed to fail. Fixing these build failures will often be a good opportunity for a first contribution, and people will always be willing to help out with this work either here or [on zulip][xi zulip].

So, in summary, the default assumption for PRs to this repo will be to merge, but this policy includes future PRs that might change or reverse stuff in previous PRs. For more information I recommend reading [the optimistic merging article linked here and above][optimistic merging], which offers an interesting approach to managing open source projects irrespective of its use here.

# Widgets

If you add a new widget, please add its name and a short summary here.

 - A tree widget
 - A Navigator widget that can display different child widgets/views.
 - Dropdown : a basic dropdown widget using the recently added sub-windows
 - Animator : a helper for running multiple animations with different curves/timing/dependencies
 - PartialWidget : a widget that shows a widget if its data is present
 - MultiRadio : a Radio that represents multiple values through an inner widget
 - MultiCheckbox : a Checkbox that represents multiple values through an inner widget

# Widget Requests

If you need a certain widget, and you think it might be useful to others, feel free to make a PR adding it to this list.

 - A really good, general widget for laying out collections of items.
   - There are different approaches we could use (`flexbox` or `grid` from the HTML world).
   - This might be multiple widgets serving different use cases in the end.
 - A widget that works like `druid::widget::Scroll` but also supports zooming its content.
 - A color picker

# Links to widget crates

Maybe you have made your own collection of widgets that aren't general enough to go into `druid` proper, but will still be useful to other `druid` users. Submit a PR to add them here!

 - [`druid-graphs`]: An alpha quality library for drawing graphs as widgets, taking inspiration from [`matplotlib`]. Currently only supports a few graph types, collaboration welcome! Works well with [`druid-lens-compose`].
 - [`druid_table`] : A table/datagrid widget (also has some interpolation/visualisation stuff currently). Uses [`druid_bindings`]

[optimistic merging]: http://hintjens.com/blog:106
[xi zulip]: https://xi.zulipchat.com/
[`druid-graphs`]: https://github.com/derekdreery/druid-graphs
[`matplotlib`]: https://matplotlib.org/
[`druid-lens-compose`]: https://github.com/derekdreery/druid-lens-compose
[`druid_table`]: https://github.com/rjwittams/druid_table/
[`druid_bindings`]: https://github.com/rjwittams/druid_bindings