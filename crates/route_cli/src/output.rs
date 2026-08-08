use language::analyze::{HTTPRoute, TCPRoute};
use serde::Serialize;

use crate::config::RouteConfig;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct K8sHTTPRoute<'a> {
    pub api_version: &'a str,
    pub kind: &'a str,
    pub metadata: Metadata<'a>,
    pub spec: HTTPRouteSpec<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct K8sTCPRoute<'a> {
    pub api_version: &'a str,
    pub kind: &'a str,
    pub metadata: Metadata<'a>,
    pub spec: TCPRouteSpec<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TCPRouteSpec<'a> {
    entry_points: Vec<&'a str>,
    routes: Vec<TCPRouteRule<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TCPRouteRule<'a> {
    pub r#match: &'a str,
    pub middlewares: Vec<TCPRouteMiddleware<'a>>,
    pub services: Vec<BackendRef<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TCPRouteMiddleware<'a> {
    pub name: &'a str,
    pub namespace: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata<'a> {
    pub name: &'a str,
    pub namespace: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HTTPRouteSpec<'a> {
    pub parent_refs: Vec<ParentRef<'a>>,
    pub hostnames: Vec<&'a str>,
    pub rules: Vec<HTTPRouteRule<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HTTPRouteRule<'a> {
    pub backend_refs: Vec<BackendRef<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<Filter<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Filter<'a> {
    pub r#type: &'a str,
    pub extension_ref: ExtensionRef<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionRef<'a> {
    pub group: &'a str,
    pub kind: &'a str,
    pub name: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendRef<'a> {
    pub name: &'a str,
    pub port: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParentRef<'a> {
    pub name: &'a str,
    pub namespace: &'a str,
}

pub fn render_output(config: &RouteConfig, http: &[HTTPRoute], tcp: &[TCPRoute]) -> String {
    let mut output = String::new();
    let private_middleware_name = &config.routes.private_middleware_name;

    for route in http {
        if !output.is_empty() {
            output.push_str("---\n");
        }

        let mut filters = Vec::new();

        if route.private {
            filters.push(Filter {
                r#type: "ExtensionRef",
                extension_ref: ExtensionRef {
                    group: "traefik.io",
                    kind: "Middleware",
                    name: private_middleware_name,
                },
            });
        }

        let result = K8sHTTPRoute {
            api_version: "gateway.networking.k8s.io/v1",
            kind: "HTTPRoute",
            metadata: Metadata {
                name: &route.name,
                namespace: &route.namespace,
            },
            spec: HTTPRouteSpec {
                parent_refs: vec![ParentRef {
                    name: &route.gateway.name,
                    namespace: &route.gateway.namespace,
                }],
                hostnames: vec![&route.hostname],
                rules: vec![HTTPRouteRule {
                    backend_refs: vec![BackendRef {
                        name: &route.service,
                        port: route.port,
                    }],
                    filters,
                }],
            },
        };
        output.push_str(&serde_yaml::to_string(&result).unwrap());
    }

    for route in tcp {
        if !output.is_empty() {
            output.push_str("---\n");
        }

        let mut middlewares = Vec::new();

        if route.private {
            middlewares.push(TCPRouteMiddleware {
                name: private_middleware_name,
                namespace: "kube-system",
            });
        }

        let result = K8sTCPRoute {
            api_version: "traefik.io/v1alpha1",
            kind: "IngressRouteTCP",
            metadata: Metadata {
                name: &route.name,
                namespace: &route.namespace,
            },
            spec: TCPRouteSpec {
                entry_points: vec![&route.entrypoint],
                routes: vec![TCPRouteRule {
                    r#match: "HostSNI(`*`)",
                    middlewares,
                    services: vec![BackendRef {
                        name: &route.service,
                        port: route.port,
                    }],
                }],
            },
        };
        output.push_str(&serde_yaml::to_string(&result).unwrap());
    }

    output
}
