extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use serde::{Deserialize, Serialize};
use syn::{parse_macro_input, Expr, FnArg, Ident, ImplItem, ImplItemMethod, ItemImpl, Stmt};

const EVENTS_INT_EXPRESSION: &str = "events_int";
const EVENTS_EXT_EXPRESSION: &str = "events_ext";

#[derive(Clone, Debug, Deserialize, Serialize)]
struct EventRule {
    event_expression: String,
    event_parameters: Vec<String>,
    event_routine: EventRoutine,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct EventRoutine {
    // State variable, value
    state_transitions: Vec<(String, String)>,
    scheduling: Vec<EventEdge>,
    cancelling: Vec<EventEdge>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct EventEdge {
    event_expression_target: String,
    parameters: Vec<String>,
    condition: Option<String>,
    delay: Option<String>,
}

enum ModelImplementation {
    DevsModel,
    Other,
}

enum DevsTransitions {
    Internal,
    External,
}

fn model_implementation(item: &ItemImpl) -> Option<ModelImplementation> {
    match &item.trait_ {
        None => None,
        Some(trait_) => {
            if trait_.1.segments[0].ident == "DevsModel" {
                Some(ModelImplementation::DevsModel)
            } else {
                Some(ModelImplementation::Other)
            }
        }
    }
}

fn get_method_args(method: &ImplItemMethod) -> Vec<String> {
    method
        .sig
        .inputs
        .iter()
        .filter_map(|input| match input {
            FnArg::Typed(pat_type) => {
                let pat = &pat_type.pat;
                Some(quote!(#pat).to_string())
            }
            FnArg::Receiver(_) => None,
        })
        .collect()
}

fn get_state_transitions(method: &ImplItemMethod) -> Vec<(String, String)> {
    method
        .block
        .stmts
        .iter()
        .filter_map(|stmt| match stmt {
            Stmt::Semi(Expr::Assign(assign), _) => {
                let assign_left = &assign.left;
                let assign_right = &assign.right;
                Some((
                    quote!(#assign_left).to_string(),
                    quote!(#assign_right).to_string(),
                ))
            }
            _ => None,
        })
        .collect()
}

fn get_schedulings(method: &ImplItemMethod) -> Option<Vec<EventEdge>> {
    method.block.stmts.iter().find_map(|stmt| {
        if let Stmt::Expr(Expr::MethodCall(method_call)) = stmt {
            Some(vec![EventEdge {
                event_expression_target: method_call.method.to_string(),
                // TODO parameters
                parameters: Vec::new(),
                condition: None,
                delay: None,
            }])
        } else if let Stmt::Expr(Expr::Match(match_)) = stmt {
            Some(
                match_
                    .arms
                    .iter()
                    .filter_map(|arm| {
                        let match_expr = &match_.expr;
                        let match_case = &arm.pat;
                        let (match_function, match_parameters) = match &*arm.body {
                            Expr::Call(call) => {
                                match &call.args[0] {
                                    Expr::MethodCall(method_call) => {
                                        // TODO - Extract function parameters
                                        (method_call.method.to_string(), Vec::new())
                                    }
                                    _ => return None,
                                }
                            }
                            Expr::MethodCall(method_call) => {
                                // TODO - Extract function parameters
                                (method_call.method.to_string(), Vec::new())
                            }
                            _ => {
                                return None;
                            }
                        };
                        Some(EventEdge {
                            event_expression_target: match_function,
                            parameters: match_parameters,
                            condition: Some(format![
                                "{} = {}",
                                quote!(#match_expr),
                                quote!(#match_case),
                            ]),
                            delay: None,
                        })
                    })
                    .collect(),
            )
        } else {
            None
        }
    })
}

fn add_event_rules_transition_method(mut input: ItemImpl) -> TokenStream {
    let mut event_rules: Vec<EventRule> = Vec::new();

    input
        .items
        .iter()
        .filter_map(|method| {
            if let ImplItem::Method(method) = method {
                Some(method)
            } else {
                None
            }
        })
        .for_each(|method| {
            let name = method.sig.ident.to_string();
            let arguments = get_method_args(method);
            let state_transitions = get_state_transitions(method);
            event_rules.push(EventRule {
                event_expression: name,
                event_parameters: arguments,
                event_routine: EventRoutine {
                    state_transitions,
                    scheduling: vec![EventEdge {
                        event_expression_target: EVENTS_INT_EXPRESSION.to_string(),
                        parameters: Vec::new(),
                        condition: None,
                        delay: Some(String::from("\\sigma")),
                    }],
                    cancelling: Vec::new(),
                },
            });
        });
    let event_rules_json = serde_json::to_string(&event_rules);
    match event_rules_json {
        Ok(event_rules_str) => {
            input.items.push(ImplItem::Verbatim(quote! {
                pub fn event_rules_transition(
                    &self,
                ) -> &str {
                    #event_rules_str
                }
            }));
            TokenStream::from(quote!(#input))
        }
        Err(err) => {
            let err_string = err.to_string();
            TokenStream::from(quote!(compile_error!(#err_string)))
        }
    }
}

fn add_event_rules_scheduling_method(mut input: ItemImpl) -> TokenStream {
    let events_int_ident = Ident::new("events_int", Span::call_site());
    let events_ext_ident = Ident::new("events_ext", Span::call_site());

    let mut event_rules: Vec<EventRule> = Vec::new();

    input
        .items
        .iter()
        .filter_map(|method| {
            if let ImplItem::Method(method) = method {
                if method.sig.ident == events_int_ident {
                    Some((DevsTransitions::Internal, method))
                } else if method.sig.ident == events_ext_ident {
                    Some((DevsTransitions::External, method))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .for_each(|(transition_type, method)| {
            let (name, cancellings) = match &transition_type {
                DevsTransitions::Internal => (EVENTS_INT_EXPRESSION.to_string(), Vec::new()),
                DevsTransitions::External => (
                    EVENTS_EXT_EXPRESSION.to_string(),
                    vec![EventEdge {
                        event_expression_target: EVENTS_INT_EXPRESSION.to_string(),
                        parameters: Vec::new(),
                        condition: None,
                        delay: None,
                    }],
                ),
            };
            let arguments = get_method_args(method);
            if let Some(schedulings) = get_schedulings(method) {
                event_rules.push(EventRule {
                    event_expression: name,
                    event_parameters: arguments,
                    event_routine: EventRoutine {
                        state_transitions: Vec::new(),
                        scheduling: schedulings,
                        cancelling: cancellings,
                    },
                });
            }
        });
    let event_rules_json = serde_json::to_string(&event_rules);
    match event_rules_json {
        Ok(event_rules_str) => {
            input.items.push(ImplItem::Verbatim(quote! {
                fn event_rules_scheduling(
                    &self,
                ) -> &str {
                    #event_rules_str
                }

                fn event_rules(&self) -> String {
                    // Avoid deserialization/serialization cycle for reduced complexity and improved performance
                    // Instead, use a manual, index-filtered string concatenation
                    let transition_str_len = self.event_rules_transition().len();
                    format![
                        "{},{}",
                        &self.event_rules_transition()[..transition_str_len-1],
                        &self.event_rules_scheduling()[1..]
                    ]
                }
            }));
            TokenStream::from(quote!(#input))
        }
        Err(err) => {
            let err_string = err.to_string();
            TokenStream::from(quote!(compile_error!(#err_string)))
        }
    }
}

#[proc_macro_attribute]
pub fn event_rules(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);

    match model_implementation(&input) {
        None => add_event_rules_transition_method(input),
        Some(model_implementation) => {
            match model_implementation {
                ModelImplementation::DevsModel => add_event_rules_scheduling_method(input),
                // (Add nothing if other trait implementations)
                ModelImplementation::Other => TokenStream::from(quote!(#input)),
            }
        }
    }
}
