use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn2::{
  parse_macro_input, parse_quote, punctuated::Punctuated, DeriveInput, Expr, Meta, Token,
  TypeParam,
};

pub fn expand_widget(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        attrs,
        mut generics,
        ..
    } = parse_macro_input!(input);

    let data_bound: TypeParam = parse_quote!(T: Clone + druid::Data);
    let widget_bound: TypeParam = parse_quote!(W: Widget<T>);
    generics
        .type_params_mut()
        .find(|param| param.ident.to_string() == "T")
        .unwrap()
        .bounds
        .extend(data_bound.bounds);
    generics
        .type_params_mut()
        .find(|param| param.ident.to_string() == "W")
        .unwrap()
        .bounds
        .extend(widget_bound.bounds);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut event = None;
    let mut lifecycle = None;
    let mut update = None;
    let mut layout = None;
    let mut paint = None;
    let mut widget_pod = None;

    let list = &attrs[0].meta.require_list().unwrap();
    assert!(list.path.is_ident("widget"));
    let args = list
        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        .unwrap();

    for args in args {
        let name_val = args.require_name_value().unwrap();

        let value = match &name_val.value {
            Expr::Path(expr_path) => {
                let ident = expr_path.path.require_ident().unwrap();
                ident.to_token_stream()
            }
            Expr::Lit(lit) => lit.to_token_stream(),
            _ => panic!(
                "Must be literal naming a method on Self {:?}",
                name_val.value
            ),
        };

        match &name_val.path {
            path if path.is_ident("event") => {
                if value.to_string() == "event" {
                    panic!("`event` method implementation cannot be named `event`")
                }
                event = Some(quote! {self.#value(ctx, event, data, env)})
            }
            path if path.is_ident("lifecycle") => {
                if value.to_string() == "lifecycle" {
                    panic!("`lifecycle` method implementation cannot be named `lifecycle`")
                }
                lifecycle = Some(quote! {self.#value(ctx, event, data, env)})
            }
            path if path.is_ident("update") => {
                if value.to_string() == "update" {
                    panic!("`update` method implementation cannot be named `update`")
                }
                update = Some(quote! {self.#value(ctx, old_data, data, env)})
            }
            path if path.is_ident("layout") => {
                if value.to_string() == "layout" {
                    panic!("`layout` method implementation cannot be named `layout`")
                }
                layout = Some(quote! {self.#value(ctx, bc, data, env)})
            }
            path if path.is_ident("paint") => {
                if value.to_string() == "paint" {
                    panic!("`paint` method implementation cannot be named `paint`")
                }
                paint = Some(quote! {self.#value(ctx, data, env)})
            }
            path if path.is_ident("widget_pod") => widget_pod = Some(quote! {self.#value}),
            _ => panic!("Must be one of `event`, `lifecycle`, `update`, `layout` or `paint`."),
        };
    }

    let widget_pod = widget_pod.unwrap_or_else(|| quote! {"self.widget_pod"});
    let event = event.unwrap_or_else(|| {
        quote! {
          #widget_pod.event(ctx, event, data, env)
        }
    });
    let lifecycle = lifecycle.unwrap_or_else(|| {
        quote! {
          #widget_pod.lifecycle(ctx, event, data, env)
        }
    });
    let update = update.unwrap_or_else(|| {
        quote! {
          #widget_pod.update(ctx, data, env)
        }
    });
    let layout = layout.unwrap_or_else(|| {
        quote! {
          #widget_pod.layout(ctx, bc, data, env)
        }
    });
    let paint = paint.unwrap_or_else(|| {
        quote! {
          #widget_pod.paint(ctx, data, env)
        }
    });

    quote! {
    impl #impl_generics druid::Widget<T> for #ident #ty_generics #where_clause {
      fn event(&mut self, ctx: &mut druid::EventCtx, event: &druid::Event, data: &mut T, env: &druid::Env) {
        #event
      }

      fn lifecycle(&mut self, ctx: &mut druid::LifeCycleCtx, event: &druid::LifeCycle, data: &T, env: &druid::Env) {
        #lifecycle
      }

      fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &T, data: &T, env: &druid::Env) {
        #update
      }

      fn layout(&mut self, ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, data: &T, env: &druid::Env) -> druid::Size {
        #layout
      }

      fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &T, env: &druid::Env) {
        #paint
      }
    }
  }.into()
}
