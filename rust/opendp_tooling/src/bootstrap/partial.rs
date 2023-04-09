use syn::{
    punctuated::Punctuated, GenericArgument, Ident, ItemFn, PathArguments, PathSegment, ReturnType,
    Signature, Token, Type,
};

use crate::bootstrap::signature::syn_fnarg_to_syn_pattype;

pub fn generate_partial(mut item_fn: ItemFn) -> Option<ItemFn> {
    if !supports_partial(&item_fn.sig) {
        return None;
    }

    // update function name
    item_fn.sig.ident = syn::Ident::new(
        &item_fn
            .sig
            .ident
            .to_string()
            .replacen("make_", "partial_", 1),
        item_fn.sig.ident.span(),
    );

    // update function arguments
    let mut inputs = Vec::from_iter(item_fn.sig.inputs.into_iter());
    let input_domain_arg = inputs.remove(0);
    let input_metric_arg = inputs.remove(0);
    item_fn.sig.inputs = Punctuated::from_iter(inputs);

    // update return type
    let syn::ReturnType::Type(_, fallible_type) = &mut item_fn.sig.output else {
        return None;
    };
    let syn::Type::Path(path) = fallible_type.as_mut() else {
        return None;
    };
    let PathArguments::AngleBracketed(args) = &mut path.path.segments.last_mut()?.arguments else {
        return None;
    };
    let GenericArgument::Type(operator_type) = &mut args.args.first_mut()? else {
        return None;
    };
    let syn::Type::Path(path) = operator_type else {
        return None;
    };

    let mut last_segment = path.path.segments.last()?.clone();
    let span = last_segment.ident.span();
    last_segment.ident = syn::Ident::new(
        format!("Partial{}", last_segment.ident.clone()).as_str(),
        last_segment.ident.span(),
    );
    path.path.segments = Punctuated::<_, Token![::]>::from_iter([
        PathSegment::from(Ident::new("crate", span)),
        PathSegment::from(Ident::new("core", span)),
        last_segment,
    ]);

    item_fn.sig.output = syn::ReturnType::Type(
        syn::token::RArrow::default(),
        Box::new(operator_type.clone()),
    );

    let old_block = item_fn.block.clone();
    // update function body
    item_fn.block = syn::parse_quote! {{
        crate::core::PartialTransformation::new(move |#input_domain_arg, #input_metric_arg| #old_block)
    }};

    Some(item_fn)
}

pub fn supports_partial(sig: &Signature) -> bool {
    if sig.inputs.len() < 2 {
        return false;
    }

    let Some((input_domain_type, input_metric_type)) = extract_domain_metric_types(&sig.output) else {
        return false;
    };

    let mut inputs = Vec::from_iter(sig.inputs.iter().cloned());

    let Ok(first_arg) = syn_fnarg_to_syn_pattype(inputs.remove(0)) else {
        return false;
    };
    let Ok(second_arg) = syn_fnarg_to_syn_pattype(inputs.remove(0)) else {
        return false;
    };

    first_arg.1 == input_domain_type && second_arg.1 == input_metric_type
}

fn extract_domain_metric_types(output: &ReturnType) -> Option<(Type, Type)> {
    let syn::ReturnType::Type(_, output_type) = output.clone() else {
        return None;
    };
    let data_type = extract_parameters(*output_type, "Fallible")?
        .first()?
        .clone();

    let supporting_types = extract_parameters(data_type.clone(), "Transformation")
        .or_else(|| extract_parameters(data_type.clone(), "Measurement"))?;

    let [input_domain_type, _, input_metric_type, _] =
        <[Type; 4]>::try_from(supporting_types).ok()?;

    Some((input_domain_type, input_metric_type))
}

fn extract_parameters(ty: Type, name: &str) -> Option<Vec<Type>> {
    let syn::Type::Path(path) = ty else {
        return None;
    };

    let segment = path.path.segments.last()?;
    if segment.ident != name {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    args.args
        .iter()
        .map(|arg| match arg {
            syn::GenericArgument::Type(ty) => Some(ty.clone()),
            _ => None,
        })
        .collect()
}
