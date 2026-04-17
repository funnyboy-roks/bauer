use proc_macro2::Group;
use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use quote::ToTokens;
use quote::TokenStreamExt;
use quote::quote;
use syn::Expr;
use syn::GenericArgument;
use syn::LitInt;
use syn::PathArguments;
use syn::ReturnType;
use syn::Type;
use syn::TypeArray;
use syn::TypeBareFn;
use syn::TypeParamBound;
use syn::TypePath;
use syn::TypeTraitObject;
use syn::TypeTuple;
use syn::parse_quote;

fn match_arrays(pattern: &TypeArray, ty: &TypeArray, out: &mut Vec<TokenStream>) -> bool {
    if pattern == ty {
        true
    } else if let Expr::Infer(_) = pattern.len {
        let ret = pattern_match_type(&pattern.elem, &ty.elem, out);
        out.push(ty.len.to_token_stream());
        ret
    } else if ty.len == pattern.len {
        pattern_match_type(&pattern.elem, &ty.elem, out)
    } else {
        false
    }
}

fn match_bare_function(pattern: &TypeBareFn, ty: &TypeBareFn, out: &mut Vec<TokenStream>) -> bool {
    if pattern == ty {
        return true;
    }

    if ty.inputs.len() != pattern.inputs.len() {
        return false;
    }

    for (f, p) in ty.inputs.iter().zip(pattern.inputs.iter()) {
        if !pattern_match_type(&p.ty, &f.ty, out) {
            return false;
        }
    }

    match (&ty.output, &pattern.output) {
        (ReturnType::Default, ReturnType::Default) => true,
        (ReturnType::Default, ReturnType::Type(_, ty)) => {
            pattern_match_type(ty, &parse_quote! { () }, out)
        }
        (ReturnType::Type(_, _), ReturnType::Default) => false,
        (ReturnType::Type(_, f), ReturnType::Type(_, p)) => pattern_match_type(p, f, out),
    }
}

fn match_path(pattern: &TypePath, ty: &TypePath, out: &mut Vec<TokenStream>) -> bool {
    if pattern == ty {
        return true;
    }

    if ty.path.segments.len() != pattern.path.segments.len() {
        return false;
    }

    for (t_seg, p_seg) in ty.path.segments.iter().zip(pattern.path.segments.iter()) {
        if t_seg.arguments.is_empty() != p_seg.arguments.is_empty() {
            return false;
        }

        match (&t_seg.arguments, &p_seg.arguments) {
            (PathArguments::None, PathArguments::None)
            | (PathArguments::None, PathArguments::AngleBracketed(_))
            | (PathArguments::None, PathArguments::Parenthesized(_))
            | (PathArguments::AngleBracketed(_), PathArguments::None)
            | (PathArguments::AngleBracketed(_), PathArguments::Parenthesized(_))
            | (PathArguments::Parenthesized(_), PathArguments::None)
            | (PathArguments::Parenthesized(_), PathArguments::AngleBracketed(_)) => {
                return false;
            }
            (PathArguments::AngleBracketed(t), PathArguments::AngleBracketed(p)) => {
                if t.args.len() != p.args.len() {
                    return false;
                }

                for (t, p) in t.args.iter().zip(p.args.iter()) {
                    match (t, p) {
                        (GenericArgument::Lifetime(tl), GenericArgument::Lifetime(pl)) => {
                            if tl != pl {
                                return false;
                            }
                        }
                        (
                            GenericArgument::Lifetime(lifetime),
                            GenericArgument::Type(Type::Infer(_)),
                        ) => {
                            out.push(lifetime.to_token_stream());
                        }
                        (GenericArgument::Lifetime(_), _) => return false,
                        (GenericArgument::Type(t_ty), GenericArgument::Type(p_ty)) => {
                            if !pattern_match_type(p_ty, t_ty, out) {
                                return false;
                            }
                        }
                        (GenericArgument::Type(_), _) => return false,
                        (GenericArgument::Const(expr), GenericArgument::Type(Type::Infer(_))) => {
                            out.push(expr.to_token_stream());
                        }
                        (GenericArgument::Const(_), _) => return false,
                        (GenericArgument::AssocType(_), _)
                        | (GenericArgument::AssocConst(_), _)
                        | (GenericArgument::Constraint(_), _) => unreachable!(),
                        _ => todo!(),
                    }
                }
            }
            (PathArguments::Parenthesized(t), PathArguments::Parenthesized(p)) => {
                if t.inputs.len() != p.inputs.len() {
                    return false;
                }

                for (f, p) in t.inputs.iter().zip(p.inputs.iter()) {
                    if !pattern_match_type(p, f, out) {
                        return false;
                    }
                }

                let out_matched = match (&t.output, &p.output) {
                    (ReturnType::Default, ReturnType::Default) => true,
                    (ReturnType::Default, ReturnType::Type(_, ty)) => {
                        pattern_match_type(ty, &parse_quote! { () }, out)
                    }
                    (ReturnType::Type(_, _), ReturnType::Default) => false,
                    (ReturnType::Type(_, f), ReturnType::Type(_, p)) => {
                        pattern_match_type(p, f, out)
                    }
                };

                if !out_matched {
                    return false;
                }
            }
        };
    }

    true
}

fn match_trait_object(
    p: &TypeTraitObject,
    t: &TypeTraitObject,
    out: &mut Vec<TokenStream>,
) -> bool {
    if p == t {
        return true;
    }

    if p.bounds.len() == 1 {
        let bound = p.bounds.iter().next().expect("checked");
        match bound {
            TypeParamBound::Trait(trait_) => {
                out.push(t.bounds.to_token_stream());
                if trait_.path.is_ident("__") {
                    return true;
                }
            }
            _ => return false,
        }
    }
    false
}

fn match_tuple(pattern: &TypeTuple, ty: &TypeTuple, out: &mut Vec<TokenStream>) -> bool {
    if pattern == ty {
        return true;
    }

    if ty.elems.len() != pattern.elems.len() {
        return false;
    }

    for (t, p) in ty.elems.iter().zip(pattern.elems.iter()) {
        if !pattern_match_type(p, t, out) {
            return false;
        }
    }

    true
}

pub fn pattern_match_type(pattern: &Type, ty: &Type, out: &mut Vec<TokenStream>) -> bool {
    if pattern == ty {
        return true;
    };

    if let Type::Infer(_) = pattern {
        out.push(ty.to_token_stream());
        return true;
    }

    match (ty, pattern) {
        (Type::Array(arr), Type::Array(pat)) => match_arrays(pat, arr, out),
        (Type::BareFn(func), Type::BareFn(pat)) => match_bare_function(pat, func, out),
        (Type::Group(t), Type::Group(p)) => pattern_match_type(&p.elem, &t.elem, out),
        (Type::Macro(t), Type::Macro(p)) => t == p,
        (Type::Ptr(t), Type::Ptr(p)) => {
            t.mutability == p.mutability && pattern_match_type(&p.elem, &t.elem, out)
        }
        (Type::Never(_), Type::Never(_)) => true,
        (Type::Paren(t), Type::Paren(p)) => pattern_match_type(&p.elem, &t.elem, out),
        (Type::Path(t), Type::Path(p)) => match_path(p, t, out),
        (Type::Reference(t), Type::Reference(p)) => {
            t.mutability == p.mutability && pattern_match_type(&p.elem, &t.elem, out)
        }
        (Type::Slice(t), Type::Slice(p)) => pattern_match_type(&p.elem, &t.elem, out),
        (Type::TraitObject(t), Type::TraitObject(p)) => match_trait_object(p, t, out),
        (Type::Tuple(t), Type::Tuple(p)) => match_tuple(p, t, out),
        (Type::ImplTrait(_), Type::ImplTrait(_)) => unreachable!("Not allowed in this position"),
        (Type::Infer(_), Type::Infer(_)) => unreachable!("Not allowed in this position"),
        (Type::Verbatim(_), Type::Verbatim(_)) => unreachable!(),
        _ => false,
    }
}

pub fn replace(matches: &[TokenStream], stream: TokenStream) -> syn::Result<TokenStream> {
    let mut out = TokenStream::new();
    let mut stream = stream.into_iter().peekable();
    while let Some(t) = stream.next() {
        let t = match t {
            TokenTree::Group(g) => {
                let stream = replace(matches, g.stream())?;
                let mut t = Group::new(g.delimiter(), stream);
                t.set_span(g.span());

                TokenTree::Group(t)
            }
            TokenTree::Ident(_) => t,
            TokenTree::Punct(ref p) if p.as_char() == '#' => {
                if let Some(peeked) = stream.peek() {
                    match peeked {
                        TokenTree::Literal(_) => {
                            let Some(TokenTree::Literal(l)) = stream.next() else {
                                unreachable!("peek was some and literal matched");
                            };

                            let int = syn::parse2::<LitInt>(l.to_token_stream())?;
                            let n = int.base10_parse::<usize>()?;

                            if let Some(m) = matches.get(n) {
                                m.to_tokens(&mut out);
                            } else {
                                let err = format!(
                                    "index out of bounds: there were {0} matches but the index is {1}, if you intended a literal '#{1}', use '# #{1}'",
                                    matches.len(),
                                    n,
                                );
                                return Err(syn::Error::new_spanned(quote! { #int #p }, err));
                            }
                            continue;
                        }
                        TokenTree::Punct(p) if p.as_char() == '#' => {
                            // munch if we have a double `#`
                            stream.next().expect("peek is some")
                        }
                        _ => t,
                    }
                } else {
                    t
                }
            }
            TokenTree::Punct(_) => t,
            TokenTree::Literal(_) => t,
        };
        out.append(t);
    }
    Ok(out)
}

#[cfg(test)]
mod test {
    use quote::quote;
    use syn::{Type, parse_quote};

    use crate::util::pattern::replace;

    use super::pattern_match_type;

    macro_rules! match_pattern {
        ($pat: ty => $($ty: ty => $($match: ty),*;)*) => {
            let pattern: Type = parse_quote! { $pat };

            let mut out = Vec::new();

            $(
                out.clear();
                let ty: Type = parse_quote! { $ty };
                let matches = pattern_match_type(&pattern, &ty, &mut out);

                for (i, m) in out.iter().enumerate() {
                    eprintln!("out[{}] = {}", i, m);
                }
                assert!(matches);

                $(
                    eprintln!("~= {}", stringify!($ty));
                    let ty: Type = syn::parse2(out.remove(0)).unwrap();
                    let inner: Type = parse_quote! { $match };
                    assert_eq!(ty, inner);
                )*
                assert!(out.is_empty(), "more matches than expected");
            )*
        };
        ($pat: ty => ! $($ty: ty),*$(,)?) => {
            let pattern: Type = parse_quote! { $pat };

            let mut out = Vec::new();

            $(
                out.clear();
                let ty: Type = parse_quote! { $ty };
                let matches = pattern_match_type(&pattern, &ty, &mut out);

                eprintln!("!= {}", stringify!($ty));
                for (i, m) in out.iter().enumerate() {
                    eprintln!("out[{}] = {}", i, m);
                }
                assert!(!matches);
            )*
        };
    }

    #[test]
    fn blanket() {
        match_pattern! {
            _ =>
                [u32; 3]              => [u32; 3];
                fn(u32, u8) -> String => fn(u32, u8) -> String;
                my_macro!(12)         => my_macro!(12);
                *const String         => *const String;
                (Vec<u32>)            => (Vec<u32>);
                Vec<u32>              => Vec<u32>;
                &dyn Foo              => &dyn Foo;
                (u32, u8)             => (u32, u8);
        };
    }

    #[test]
    fn vec_underscore() {
        match_pattern! {
            Vec<_> =>
                Vec<u32>              => u32;
        }
        match_pattern! {
            Vec<_> => !
                [u32; 3],
                fn(u32, u8) -> String,
                my_macro!(12),
                *const String,
                (Vec<u32>),
                &dyn Foo,
                (u32, u8),
        };
    }

    #[test]
    fn arrays() {
        match_pattern! {
            [_; 3] =>
                [u32; 3]    => u32;
                [String; 3] => String;
        }
        match_pattern! {
            [_; 3] => !
                [u32; 4],
                fn(u32, u8) -> String,
                my_macro!(12),
                *const String,
                (Vec<u32>),
                &dyn Foo,
                (u32, u8),
        };
    }

    #[test]
    fn bare_fn_arg() {
        match_pattern! {
            fn(_) -> u32 =>
                fn(u64) -> u32    => u64;
                fn(String) -> u32 => String;
        }
        match_pattern! {
            fn(_) -> u32 => !
                [u32; 4],
                my_macro!(12),
                *const String,
                (Vec<u32>),
                &dyn Foo,
                (u32, u8),
        };
    }

    #[test]
    fn bare_fn_output() {
        match_pattern! {
            fn(u32) -> _ =>
                fn(u32) -> u64    => u64;
                fn(u32) -> String => String;
        }
        match_pattern! {
            fn(u32) -> _ => !
                [u32; 4],
                my_macro!(12),
                *const String,
                (Vec<u32>),
                &dyn Foo,
                (u32, u8),
        };
    }

    #[test]
    fn pointers() {
        match_pattern! {
            *const _ =>
                *const String    => String;
                *const *const u8 => *const u8;
                *const *mut u8   => *mut u8;
        }
        match_pattern! {
            *const _ => !
                [u32; 3],
                fn(u32, u8) -> String,
                my_macro!(12),
                *mut String,
                (Vec<u32>),
                &dyn Foo,
                (u32, u8),
        };
    }

    #[test]
    fn paren() {
        match_pattern! {
            (_) =>
                ([u32; 3])              => [u32; 3];
                (fn(u32, u8) -> String) => fn(u32, u8) -> String;
                (my_macro!(12))         => my_macro!(12);
                (*const String)         => *const String;
                ((Vec<u32>))            => (Vec<u32>);
                (Vec<u32>)              => Vec<u32>;
                (&dyn Foo)              => &dyn Foo;
                ((u32, u8))             => (u32, u8);
        }
        match_pattern! {
            (_) => !
                [u32; 3],
                fn(u32, u8) -> String,
                my_macro!(12),
                *const String,
                Vec<u32>,
                &dyn Foo,
                (u32, u8),
        };
    }

    #[test]
    fn reference() {
        match_pattern! {
            &_ =>
                &[u32; 3]              => [u32; 3];
                &fn(u32, u8) -> String => fn(u32, u8) -> String;
                &my_macro!(12)         => my_macro!(12);
                &*const String         => *const String;
                &(Vec<u32>)            => (Vec<u32>);
                &Vec<u32>              => Vec<u32>;
                &dyn Foo               => dyn Foo;
                &(u32, u8)             => (u32, u8);
        }
        match_pattern! {
            &_ => !
                [u32; 3],
                fn(u32, u8) -> String,
                my_macro!(12),
                *const String,
                Vec<u32>,
                dyn Foo,
                (u32, u8),
        };
    }

    #[test]
    fn slice() {
        match_pattern! {
            [_] =>
                [[u32; 3]]              => [u32; 3];
                [fn(u32, u8) -> String] => fn(u32, u8) -> String;
                [my_macro!(12)]         => my_macro!(12);
                [*const String]         => *const String;
                [(Vec<u32>)]            => (Vec<u32>);
                [Vec<u32>]              => Vec<u32>;
                [dyn Foo]               => dyn Foo;
                [(u32, u8)]             => (u32, u8);
        }
        match_pattern! {
            [_] => !
                [u32; 3],
                fn(u32, u8) -> String,
                my_macro!(12),
                *const String,
                Vec<u32>,
                dyn Foo,
                (u32, u8),
        };
    }

    #[test]
    fn trait_object() {
        match_pattern! {
            dyn __ =>
                dyn Foo       => Foo;
                dyn Foo + Bar => Foo + Bar;
        }
        match_pattern! {
            dyn __ => !
                [u32; 3],
                fn(u32, u8) -> String,
                my_macro!(12),
                *const String,
                Vec<u32>,
                &dyn Foo,
                (u32, u8),
        };
    }

    #[test]
    fn tuple_single() {
        match_pattern! {
            (_, u32) =>
                (String, u32) => String;
                (u8, u32) => u8;
        }
        match_pattern! {
            dyn _T => !
                [u32; 3],
                fn(u32, u8) -> String,
                my_macro!(12),
                *const String,
                Vec<u32>,
                &dyn Foo,
                (u8, u8),
                (u8, u32, String),
        };
    }

    #[test]
    fn tuple_multiple() {
        match_pattern! {
            (_, _) =>
                (String, u32) => String, u32;
                (u8, u32) => u8, u32;
        }
        match_pattern! {
            dyn __ => !
                [u32; 3],
                fn(u32, u8) -> String,
                my_macro!(12),
                *const String,
                Vec<u32>,
                &dyn Foo,
                (u8, u32, String),
        };
    }

    macro_rules! test_replace {
        ([$({$($matches: tt)*}),*$(,)?] + { $($tt: tt)* } => $($out: tt)*) => {
            eprintln!("{} + {} => {}", stringify!([$({$($matches)*}),*]), stringify!($($tt)*), stringify!($($out)*));
            let x = replace(&[$(quote! { $($matches)* }),*], quote! { $($tt)* }).unwrap();
            assert_eq!(x.to_string(), quote! { $($out)* }.to_string());
        };
    }

    #[test]
    fn replace_() {
        test_replace! {
            [{ foo }] + { Foo<#0> } => Foo<foo>
        };
        test_replace! {
            [{ foo }, { bar }] + { Foo<#0, #1> } => Foo<foo, bar>
        };
    }

    #[test]
    fn roundtrip() {
        let mut out = Vec::new();
        let matches = pattern_match_type(
            &parse_quote! { HashMap<_, _> },
            &parse_quote! { HashMap<String, Value> },
            &mut out,
        );
        assert!(matches);

        let out = replace(
            &out,
            quote! { repeat = (#0, #1), adapter = |name: impl Into<#0>, value: Value| (name.into(), value) },
        ).unwrap();
        dbg!(out.to_string());
        assert_eq!(
            out.to_string(),
            quote! {  repeat = (String, Value), adapter = |name: impl Into<String>, value: Value| (name.into(), value)  }.to_string()
        );
    }
}
