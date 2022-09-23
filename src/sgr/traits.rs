use std::borrow::Cow;
use proc_macro2::TokenStream;
use quote::quote;
use super::base::Output;


pub trait SgrCode {
    fn params(&self) -> Option<Cow<str>>;

    fn render(&self) -> String {
        match self.params() {
            Some(params) => format!(sgr!("{}"), params),
            None => String::new(),
        }
    }
}

impl SgrCode for String {
    fn params(&self) -> Option<Cow<str>> {
        Some(Cow::Borrowed(self))
    }
}


pub trait SgrData {
    type CodeOpening: SgrCode;
    type CodeClosing: SgrCode;

    fn fmt_opening(&self) -> Self::CodeOpening;
    fn fmt_closing(&self) -> Self::CodeClosing;

    fn contents(&self) -> TokenStream;
    fn template(&self) -> Option<syn::LitStr>;
    fn output(&self) -> Output;

    fn tokens(&self) -> TokenStream {
        let fmt: String = self.fmt_opening().render();
        let end: String = self.fmt_closing().render();

        let contents = self.contents();
        let template = self.template();

        match self.output() {
            Output::Concat => {
                assert!(template.is_none());
                quote!(concat!(concat!(#fmt, #contents), #end))
            }
            Output::ConstFormat => {
                let template = template.as_ref().unwrap();
                let temp_fmt = format!("{fmt}{}{end}", template.value());
                let temp_lit = syn::LitStr::new(&temp_fmt, template.span());

                quote!(::const_format::formatcp!(#temp_lit, #contents))
            }
            Output::Format => {
                let template = template.as_ref().unwrap();
                let temp_fmt = format!("{fmt}{}{end}", template.value());
                let temp_lit = syn::LitStr::new(&temp_fmt, template.span());

                quote!(format_args!(#temp_lit, #contents))
            }
            Output::String => {
                let template = template.as_ref().unwrap();
                let temp_fmt = format!("{fmt}{}{end}", template.value());
                let temp_lit = syn::LitStr::new(&temp_fmt, template.span());

                quote!(format!(#temp_lit, #contents))
            }
        }
    }
}
