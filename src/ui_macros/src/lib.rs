use derive_syn_parse::Parse;
use heck::CamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use syn::parse::ParseBuffer;
use syn::token;
use syn::{parse_macro_input, parse_quote, Result, Token};

#[derive(Debug, Parse)]
struct Attr {
    key: syn::Ident,
    eq_token: Token![=],
    #[brace]
    value_brace: token::Brace,
    #[inside(value_brace)]
    value: syn::Expr,
}

fn parse_attrs(buffer: &ParseBuffer) -> Result<Vec<Attr>> {
    let mut tokens = Vec::new();
    while !(buffer.peek(Token![>]) || buffer.peek(Token![/])) {
        let token = buffer.parse()?;
        tokens.push(token);
    }

    Ok(tokens)
}

#[derive(Debug, Parse)]
struct Element {
    lt_token: Token![<],
    archetype: syn::Ident,
    #[call(parse_attrs)]
    attrs: Vec<Attr>,
    suffix: ElementSuffix,
    gt_token: Token![>],
}

#[derive(Debug, Parse)]
enum ElementSuffix {
    #[peek(Token![>], name = "element with children")]
    WithChildren(WithChildren),

    #[peek(Token![/], name = "element without children")]
    WithoutChildren(Token![/]),
}

#[derive(Debug, Parse)]
struct ForLoop {
    for_token: Token![for],
    pat: syn::Pat,
    in_token: Token![in],
    #[call(syn::Expr::parse_without_eager_brace)]
    expr: syn::Expr,
    #[brace]
    brace_token: token::Brace,
    #[inside(brace_token)]
    body: Box<Item>,
}

#[derive(Debug, Parse)]
enum Item {
    #[peek(Token![<], name = "element")]
    Element(Element),

    #[peek(Token![for], name = "for loop")]
    ForLoop(ForLoop),
}

fn parse_children(buffer: &ParseBuffer) -> Result<Vec<Item>> {
    let mut tokens = Vec::new();
    while !(buffer.peek(Token![<]) && buffer.peek2(Token![/])) {
        let token = buffer.parse()?;
        tokens.push(token);
    }

    Ok(tokens)
}

#[derive(Debug, Parse)]
struct WithChildren {
    gt_token: Token![>],
    #[call(parse_children)]
    children: Vec<Item>,
    lt_token: Token![<],
    slash_token: Token![/],
    close_archetype: syn::Ident,
}

#[derive(Default)]
struct Visitor {
    decls: HashMap<BTreeSet<syn::Ident>, (syn::Ident, TokenStream)>,
}

impl Visitor {
    fn visit_element(&mut self, element: Element) -> Result<TokenStream> {
        let Element {
            lt_token: _,
            archetype,
            attrs,
            suffix,
            gt_token: _,
        } = element;

        let archetype: syn::Ident = syn::parse_str(&archetype.to_string().to_camel_case()).unwrap();

        let mut attrs: BTreeMap<String, syn::Expr> = attrs
            .into_iter()
            .map(|attr| (attr.key.to_string(), attr.value))
            .collect();

        let path_segment_expr = attrs.remove("key").map_or_else(
            || {
                quote! {
                    {
                        counter += 1;
                        WidgetPathSegment::Ordinal(counter)
                    }
                }
            },
            |key_expr| {
                quote! {
                    WidgetPathSegment::Key(#key_expr)
                }
            },
        );

        let (prop_ty, prop_expr): (Vec<syn::Ident>, Vec<syn::Expr>) = attrs
            .into_iter()
            .map(|(key, value)| {
                let prop_ty = syn::parse_str(&key.to_camel_case()).unwrap();
                let prop_expr = parse_quote! { <#prop_ty as Property>::Value::from(#value) };
                (prop_ty, prop_expr)
            })
            .unzip();

        let child_exprs = match suffix {
            ElementSuffix::WithChildren(with_children) => with_children
                .children
                .into_iter()
                .map(|item| self.visit_item(item))
                .collect::<Result<Vec<_>>>()?,

            ElementSuffix::WithoutChildren(_) => Vec::new(),
        };

        let children_expr = if child_exprs.is_empty() {
            quote! { Vec::new() }
        } else {
            quote! { <[_]>::into_vec(Box::new([ #( Box::new(#child_exprs), )* ])) }
        };

        let ty = self.decls.len();

        let (ty, _) = self
            .decls
            .entry(prop_ty.clone().into_iter().collect())
            .or_insert_with(|| {
                let ty = format_ident!("Element{}", ty);

                let decls = quote! {
                    #[allow(non_snake_case)]
                    struct #ty<W> {
                        archetype: Rc<W>,
                        id: WidgetId,
                        children: Vec<Box<dyn ExtendPropertyMap>>,
                        #( #prop_ty: <#prop_ty as Property>::Value, )*
                    }

                    impl<W> #ty<W>
                    where
                        W: Widget + 'static
                    {
                        fn render(&self, map: &mut PropertyMap) {
                            let id = self.id;
                            map.insert(id, Archetype, self.archetype.clone());
                            #( map.insert(id, <#prop_ty as Default>::default(), self.#prop_ty.clone()); )*

                            for child in self.children.iter() {
                                child.extend_property_map(id, map);
                            }
                        }
                    }

                    impl<W> IntoPropertyMap for #ty<W>
                    where
                        W: Widget + 'static,
                        #( W: HasProperty<#prop_ty>, )*
                    {
                        fn into_property_map(self, map: &mut PropertyMap) -> WidgetId {
                            self.render(map);
                            self.id
                        }
                    }

                    impl<W> ExtendPropertyMap for #ty<W>
                    where
                        W: Widget + 'static,
                        #( W: HasProperty<#prop_ty>, )*
                    {
                        fn extend_property_map(&self, parent_id: WidgetId, map: &mut PropertyMap) {
                            map.insert(self.id, Parent, parent_id);
                            self.render(map);
                        }
                    }
                };

                (ty, decls)
            });

        Ok(quote! {
            {
                let mut path = path.clone();
                path.push(#path_segment_expr);

                let me = db.intern_widget_path(path.clone());

                #ty {
                    archetype: Rc::new(<#archetype as Default>::default()),
                    id: me,
                    children: #children_expr,
                    #( #prop_ty: #prop_expr, )*
                }
            }
        })
    }

    fn visit_for_loop(&mut self, for_loop: ForLoop) -> Result<TokenStream> {
        let ForLoop {
            for_token: _,
            pat,
            in_token: _,
            expr,
            brace_token: _,
            body,
        } = for_loop;

        let body_expr = self.visit_item(*body)?;

        Ok(quote! {
            IntoIterator::into_iter(#expr).map(|#pat| #body_expr).collect::<Vec<_>>()
        })
    }

    fn visit_item(&mut self, item: Item) -> Result<TokenStream> {
        match item {
            Item::Element(item) => self.visit_element(item),
            Item::ForLoop(item) => self.visit_for_loop(item),
        }
    }
}

fn render_impl(input: Item) -> Result<TokenStream> {
    let mut visitor = Visitor::default();
    let expr = visitor.visit_item(input)?;
    let Visitor { decls } = visitor;
    let mut decls: Vec<(syn::Ident, TokenStream)> = decls.into_iter().map(|(_, value)| value).collect();
    decls.sort_by(|(ty1, _), (ty2, _)| ty1.cmp(ty2));

    let decls: Vec<TokenStream> = decls.into_iter().map(|(_, decl)| decl).collect();
    Ok(quote! {
        {
            use ui::prelude::*;

            #( #decls )*

            let path = Vec::new();
            let mut counter = 0;
            #expr
        }
    })
}

#[proc_macro]
pub fn render(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Item);
    render_impl(input).map_or_else(|e| e.into_compile_error().into(), |t| t.into())
}
