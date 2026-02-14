use quote::quote;

pub enum Segment {
    Literal(String),
    Variable(String),
}

pub fn parse_interpolation(text: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut rest = text;

    while let Some(start) = rest.find("${") {
        if start > 0 {
            segments.push(Segment::Literal(rest[..start].to_string()));
        }
        let after = &rest[start + 2..];
        let end = after.find('}').expect("unclosed ${...} in text content");
        let var_name = after[..end].trim();
        assert!(!var_name.is_empty(), "empty variable name in interpolation");
        segments.push(Segment::Variable(var_name.to_string()));
        rest = &after[end + 1..];
    }

    if !rest.is_empty() {
        segments.push(Segment::Literal(rest.to_string()));
    }
    segments
}

pub fn generate_text_arg(content: &str) -> proc_macro2::TokenStream {
    let segments = parse_interpolation(content);

    // No interpolation — plain string literal
    if segments.iter().all(|s| matches!(s, Segment::Literal(_))) {
        return quote! { #content };
    }

    // Single variable, no surrounding text
    if segments.len() == 1 {
        if let Segment::Variable(ref path) = segments[0] {
            let field: syn::Expr = syn::parse_str(&format!("&self.{}", path))
                .expect("invalid variable path in ${...}");
            return quote! { #field };
        }
    }

    // Mixed content — build a format!() call
    let mut fmt_str = String::new();
    let mut args: Vec<proc_macro2::TokenStream> = Vec::new();
    for seg in &segments {
        match seg {
            Segment::Literal(s) => {
                // Escape braces for format!()
                fmt_str.push_str(&s.replace('{', "{{").replace('}', "}}"));
            }
            Segment::Variable(path) => {
                fmt_str.push_str("{}");
                let field: syn::Expr = syn::parse_str(&format!("self.{}", path))
                    .expect("invalid variable path in ${...}");
                args.push(quote! { #field });
            }
        }
    }
    quote! { format!(#fmt_str, #(#args),*) }
}
