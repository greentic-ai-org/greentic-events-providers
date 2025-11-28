use greentic_types::TenantCtx;

/// Render a canonical tenant string "<environment>/<tenant>/<team_or_underscore>".
pub fn tenant_key(tenant: &TenantCtx) -> String {
    let team = tenant
        .team_id
        .as_ref()
        .or(tenant.team.as_ref())
        .map(|t| t.as_str())
        .unwrap_or("_");
    format!(
        "{}/{}/{}",
        tenant.env.as_str(),
        tenant.tenant.as_str(),
        team
    )
}

/// Default secret key for a provider under the greentic-secrets convention.
pub fn events_provider_secret_key(tenant: &TenantCtx, provider_name: &str) -> String {
    format!(
        "events/{}/{}/credentials",
        provider_name,
        tenant_key(tenant)
    )
}
