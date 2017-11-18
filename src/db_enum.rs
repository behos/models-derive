//! DBEnum provides diesel implementations for using an enum as a string field
//! in models. Deriving DBEnum in an enum provides serializationa and
//! deserialization traits.


use quote::Tokens;
use syn::{Ident, Variant, MacroInput};
use syn::Body::Enum;


pub fn impl_db_enum(ast: &MacroInput) -> Tokens {
    let name = &ast.ident;
    match ast.body {
        Enum(ref variants) => impl_diesel_traits(name, variants),
        _ => panic!("Doesn't work with structs"),
    }
}

fn impl_diesel_traits(name: &Ident, variants: &[Variant]) -> Tokens {
    let value_matcher_read = impl_value_matcher_read(name, variants);
    let value_matcher_write = impl_value_matcher_write(name, variants);

    let backend = quote!(::diesel::backend::Backend);
    let from_sql = quote!(::diesel::types::FromSql);
    let to_sql = quote!(::diesel::types::ToSql);
    let to_sql_output = quote!(::diesel::types::ToSqlOutput);
    let error = quote!(Box<::std::error::Error+Send+Sync>);
    let text = quote!(::diesel::types::Text);
    let write = quote!(::std::io::Write);
    let is_null = quote!(::diesel::types::IsNull);

    quote! {

        impl<DB: #backend<RawValue=[u8]>> #from_sql<#text, DB> for #name {
            fn from_sql(value: Option<&[u8]>) -> Result<Self, #error> {
                #value_matcher_read
            }
        }

        impl<DB: #backend> #to_sql<#text, DB> for #name {
            fn to_sql<W: #write>(
                &self, output: &mut #to_sql_output<W, DB>
            ) -> Result<#is_null, #error> {
                let write_string = #value_matcher_write;
                #to_sql::<#text, DB>::to_sql(write_string, output)
            }
        }
        
        use ::diesel::types::Text as DBEnumText;

        queryable_impls!(DBEnumText -> #name);
        expression_impls!(DBEnumText -> #name);
    }
}

fn impl_value_matcher_read(name: &Ident, variants: &[Variant]) -> Tokens {
    let error_handlers = impl_error_handlers();
    let variant_handlers = variants.iter().map(|v| {
        let variant_name = &v.ident;
        quote!(Ok(stringify!(#variant_name)) => Ok(#name::#variant_name))
    });

    let from_utf8 = quote!(::std::str::from_utf8);

    quote!(
        match value {
            Some(enum_string) => match #from_utf8(enum_string) {
                #(#variant_handlers,)*
                #error_handlers
            },
            None => Err(From::from("Value not provided"))
        }
    )
}

fn impl_value_matcher_write(name: &Ident, variants: &[Variant]) -> Tokens {
    let variant_handlers = variants.iter().map(|v| {
        let variant_name = &v.ident;
        quote!(&#name::#variant_name => stringify!(#variant_name))
    });

    quote!(
        match self {
            #(#variant_handlers,)*
        }
    )
}


fn impl_error_handlers() -> Tokens {
    quote!(
        Ok(_) => Err(From::from("Unexpected value")),
        Err(_) => Err(From::from("Could not load string"))
    )
}
