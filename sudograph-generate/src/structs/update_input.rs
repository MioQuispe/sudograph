// TODO once clean, make the create_input.rs file look and work the same
// TODO we should only pass inputs into sudodb if they are there
// TODO I believe that in the create_input.rs file we will need to start using MaybeUndefined

use crate::{
    get_graphql_type_name,
    is_graphql_type_a_relation_many,
    is_graphql_type_a_relation_one,
    is_graphql_type_nullable,
    structs::object_type::get_rust_type_for_object_type_named_type
};
use graphql_parser::schema::{
    Document,
    Field,
    ObjectType,
    Type
};
use proc_macro2::{
    Ident,
    TokenStream
};
use quote::{
    format_ident,
    quote
};

pub fn generate_update_input_rust_structs(
    graphql_ast: &Document<String>,
    object_types: &Vec<ObjectType<String>>
) -> Vec<TokenStream> {
    let update_input_rust_structs = object_types.iter().map(|object_type| {
        return generate_update_input_rust_struct(
            graphql_ast,
            object_type
        );
    }).collect();

    return update_input_rust_structs;
}

fn generate_update_input_rust_struct(
    graphql_ast: &Document<String>,
    object_type: &ObjectType<String>
) -> TokenStream {
    let update_input_rust_struct_name = generate_update_input_rust_struct_name(object_type);
    let update_input_rust_struct_fields = generate_update_input_rust_struct_fields(
        graphql_ast,
        object_type
    );
    let update_field_input_pushers = generate_update_field_input_pushers(
        graphql_ast,
        object_type
    );
    let update_input_rust_struct = quote! {
        #[derive(InputObject)]
        struct #update_input_rust_struct_name {
            #(#update_input_rust_struct_fields),*
        }

        impl #update_input_rust_struct_name {
            fn get_update_field_inputs(&self) -> Vec<FieldInput> {
                // TODO do this immutably if possible
                let mut update_field_inputs = vec![];

                #(#update_field_input_pushers)*
                
                return update_field_inputs;
            }
        }
    };

    return update_input_rust_struct;
}

fn generate_update_input_rust_struct_name(object_type: &ObjectType<String>) -> Ident {
    return format_ident!(
        "{}",
        String::from("Update") + &object_type.name + "Input"
    );
}

fn generate_update_input_rust_struct_fields(
    graphql_ast: &Document<String>,
    object_type: &ObjectType<String>
) -> Vec<TokenStream> {
    return object_type.fields.iter().map(|field| {
        return generate_update_input_rust_struct_field(
            graphql_ast,
            field
        );
    }).collect();
}

fn generate_update_input_rust_struct_field(
    graphql_ast: &Document<String>,
    field: &Field<String>
) -> TokenStream {
    let update_input_rust_struct_field_name_string = &field.name;
    let update_input_rust_struct_field_name = format_ident!(
        "{}",
        field.name
    );
    let update_input_rust_struct_field_rust_type = get_update_input_rust_struct_field_rust_type(
        graphql_ast,
        String::from(&field.name),
        &field.field_type,
        false
    );

    return quote! {
        #[graphql(name = #update_input_rust_struct_field_name_string)]
        #update_input_rust_struct_field_name: #update_input_rust_struct_field_rust_type
    };
}

fn get_update_input_rust_struct_field_rust_type(
    graphql_ast: &Document<String>,
    update_input_rust_struct_field_name: String,
    update_input_rust_struct_field_type: &Type<String>,
    is_non_null_type: bool
) -> TokenStream {
    match update_input_rust_struct_field_type {
        Type::NamedType(named_type) => {
            let rust_type_for_named_type = get_rust_type_for_object_type_named_type(
                graphql_ast,
                update_input_rust_struct_field_type,
                named_type
            );

            if is_graphql_type_a_relation_many(graphql_ast, update_input_rust_struct_field_type) == true {
                return quote! { MaybeUndefined<UpdateRelationManyInput> }; // TODO I do not think this would ever happen
            }
            else if is_graphql_type_a_relation_one(graphql_ast, update_input_rust_struct_field_type) == true {
                if is_non_null_type == true {
                    return quote! { MaybeUndefined<UpdateNonNullableRelationOneInput> };
                }
                else {
                    return quote! { MaybeUndefined<UpdateNullableRelationOneInput> };
                }
            }
            else {
                if update_input_rust_struct_field_name == "id" { // TODO elsewhere this check was not doing what I thought it was
                    return quote! { #rust_type_for_named_type };
                }
                else {
                    return quote! { MaybeUndefined<#rust_type_for_named_type> };
                }
            }
        },
        Type::NonNullType(non_null_type) => {
            let update_input_rust_struct_field_rust_type = get_update_input_rust_struct_field_rust_type(
                graphql_ast,
                update_input_rust_struct_field_name,
                non_null_type,
                true
            );

            return quote! { #update_input_rust_struct_field_rust_type };
        },
        Type::ListType(_) => {
            return quote! { MaybeUndefined<UpdateRelationManyInput> };
        }
    };
}

fn generate_update_field_input_pushers(
    graphql_ast: &Document<String>,
    object_type: &ObjectType<String>
) -> Vec<TokenStream> {
    let update_field_input_pushers = object_type.fields.iter().filter_map(|field| {
        if field.name == "id" {
            return None;
        }
        else {
            if is_graphql_type_a_relation_many(graphql_ast, &field.field_type) == true {
                return Some(generate_update_field_input_pusher_for_relation_many(field));
            }

            if is_graphql_type_a_relation_one(graphql_ast, &field.field_type) == true {
                return Some(generate_update_field_input_pusher_for_relation_one(field));
            }

            return Some(generate_update_field_input_pusher_for_scalar(field));
        }
    }).collect();

    return update_field_input_pushers;
}

fn generate_update_field_input_pusher_for_relation_many(field: &Field<String>) -> TokenStream {
    let field_name_string = &field.name;         
    let field_name_ident = format_ident!(
        "{}",
        field.name
    );
    let relation_object_type_name = get_graphql_type_name(&field.field_type);

    return quote! {
        match &self.#field_name_ident {
            MaybeUndefined::Value(value) => {
                if let Some(connect) = &value.connect {
                    update_field_inputs.push(FieldInput {
                        field_name: String::from(#field_name_string),
                        field_value: FieldValue::RelationMany(Some(FieldValueRelationMany {
                            relation_object_type_name: String::from(#relation_object_type_name),
                            relation_primary_keys: connect.iter().map(|id| {
                                return id.to_string();
                            }).collect(),
                            relation_primary_keys_to_remove: vec![]
                        }))
                    });
                }

                if let Some(disconnect) = &value.disconnect {
                    update_field_inputs.push(FieldInput {
                        field_name: String::from(#field_name_string),
                        field_value: FieldValue::RelationMany(Some(FieldValueRelationMany {
                            relation_object_type_name: String::from(#relation_object_type_name),
                            relation_primary_keys: vec![],
                            relation_primary_keys_to_remove: disconnect.iter().map(|id| {
                                return id.to_string();
                            }).collect()
                        }))
                    });
                }
            },
            _ => ()
        };
    };
}

fn generate_update_field_input_pusher_for_relation_one(
    field: &Field<String>
) -> TokenStream {
    let field_name_string = &field.name;         
    let field_name_ident = format_ident!(
        "{}",
        field.name
    );
    let relation_object_type_name = get_graphql_type_name(&field.field_type);

    // TODO I am not sure if we can interpolate based on a boolean, if so we could probably simplify this
    if is_graphql_type_nullable(&field.field_type) == true {
        return quote! {
            match &self.#field_name_ident {
                MaybeUndefined::Value(value) => {
                    if let Some(connect) = &value.connect {
                        update_field_inputs.push(FieldInput {
                            field_name: String::from(#field_name_string),
                            field_value: FieldValue::RelationOne(Some(FieldValueRelationOne {
                                relation_object_type_name: String::from(#relation_object_type_name),
                                relation_primary_key: connect.to_string()
                            }))
                        });
                    }
    
                    if let Some(disconnect) = &value.disconnect {
                        update_field_inputs.push(FieldInput {
                            field_name: String::from(#field_name_string),
                            field_value: FieldValue::RelationOne(None)
                        });
                    }
                },
                MaybeUndefined::Null => {
                    update_field_inputs.push(FieldInput {
                        field_name: String::from(#field_name_string),
                        field_value: FieldValue::RelationOne(None)
                    });
                },
                _ => ()
            };
        };
    }
    else {
        return quote! {
            match &self.#field_name_ident {
                MaybeUndefined::Value(value) => {
                    update_field_inputs.push(FieldInput {
                        field_name: String::from(#field_name_string),
                        field_value: FieldValue::RelationOne(Some(FieldValueRelationOne {
                            relation_object_type_name: String::from(#relation_object_type_name),
                            relation_primary_key: value.connect.to_string()
                        }))
                    });
                },
                _ => ()
            };
        };
    }
}

fn generate_update_field_input_pusher_for_scalar(field: &Field<String>) -> TokenStream {
    let field_name_string = &field.name;         
    let field_name_ident = format_ident!(
        "{}",
        field.name
    );

    return quote! {
        match &self.#field_name_ident {
            MaybeUndefined::Value(value) => {
                update_field_inputs.push(FieldInput {
                    field_name: String::from(#field_name_string),
                    field_value: value.sudo_serialize()
                });
            },
            MaybeUndefined::Null => {
                update_field_inputs.push(FieldInput {
                    field_name: String::from(#field_name_string),
                    field_value: FieldValue::Scalar(None)
                });
            },
            MaybeUndefined::Undefined => ()
        };
    };
}