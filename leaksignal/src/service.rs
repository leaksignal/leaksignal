use indexmap::IndexMap;
use url::Url;

use crate::root::DYN_ENVIRONMENT;

fn parse_spiffe_id(url: &Url) -> Option<Vec<&str>> {
    if url.scheme() != "spiffe" {
        return None;
    }
    let host = url.host_str()?;
    let mut out = vec![host];
    out.extend(url.path_segments()?);
    Some(out)
}

fn parse_service_name(san: &str) -> String {
    let peer_san = Url::parse(san).ok();
    let peer_san = peer_san.as_ref().and_then(parse_spiffe_id);
    if let Some(spiffe_id) = peer_san {
        // check for istio format
        if spiffe_id.len() == 5 && spiffe_id[1] == "ns" && spiffe_id[3] == "sa" {
            format!("{}/{}/{}", spiffe_id[0], spiffe_id[2], spiffe_id[4])
        } else {
            spiffe_id.join("/")
        }
    } else {
        san.to_string()
    }
}

fn extract_istio_service_from_env(env: &IndexMap<String, String>) -> Option<String> {
    Some(format!(
        "{}/{}/{}",
        env.get("ISTIO_META_MESH_ID")
            .or_else(|| env.get("TRUST_DOMAIN"))?,
        env.get("POD_NAMESPACE")?,
        env.get("ISTIO_META_WORKLOAD_NAME")
            .or_else(|| env.get("SERVICE_ACCOUNT"))?
    ))
}

fn extract_k8s_service_from_env(env: &IndexMap<String, String>) -> Option<String> {
    let pod_name = env.get("POD_NAME")?;
    let parts = pod_name.split('-').collect::<Vec<_>>();
    let pod_name = parts[..parts.len().saturating_sub(2)].join("-");
    Some(format!(
        "{}/{}/{}",
        env.get("POD_NAMESPACE")?,
        env.get("SERVICE_ACCOUNT")?,
        pod_name
    ))
}

fn try_get_any(
    mut get_property: impl FnMut(&str) -> Option<String>,
    keys: &[&str],
) -> Option<String> {
    keys.iter().find_map(|k| get_property(k))
}

pub fn outbound_peer_service_name(
    mut get_property: impl FnMut(&str) -> Option<String>,
) -> Option<String> {
    get_property("connection.uri_san_peer_certificate").map(|san| parse_service_name(&san))
}

#[allow(unused)]
pub fn peer_service_name(get_property: impl FnMut(&str) -> Option<String>) -> Option<String> {
    try_get_any(
        get_property,
        &[
            "connection.uri_san_peer_certificate",
            "upstream.uri_san_peer_certificate",
        ],
    )
    .map(|san| parse_service_name(&san))
}

pub fn local_service_name(get_property: impl FnMut(&str) -> Option<String>) -> Option<String> {
    let environment = DYN_ENVIRONMENT.load();
    if let Some(san) = try_get_any(
        get_property,
        &[
            "connection.uri_san_local_certificate",
            "upstream.uri_san_local_certificate",
        ],
    ) {
        Some(parse_service_name(&san))
    } else if let Some(istio_service) = extract_istio_service_from_env(&environment) {
        Some(istio_service)
    } else {
        extract_k8s_service_from_env(&environment)
    }
}

#[allow(unused)]
pub fn is_response_outbound(mut get_property: impl FnMut(&str) -> Option<String>) -> bool {
    if get_property("connection.uri_san_local_certificate").is_some() {
        true
    } else {
        get_property("upstream.uri_san_local_certificate").is_none()
    }
}
