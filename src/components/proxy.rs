use leptos::prelude::*;
use crate::proxy::*;
use crate::containers::list_containers;
use crate::components::icons::{Globe, Lock, Plus, Trash, Gear};

// ─── SSL Status Badge ─────────────────────────────────────────────────────────

#[component]
fn SslBadge(status: String, enabled: bool) -> impl IntoView {
    let (bg, label) = if !enabled {
        ("bg-gray-500/10 text-gray-500 border-gray-500/20", "No SSL")
    } else {
        match status.as_str() {
            "active"              => ("bg-green-500/10 text-green-400 border-green-500/20", "SSL Active"),
            "provisioning"        => ("bg-blue-500/10 text-blue-400 border-blue-500/20 animate-pulse", "Provisioning"),
            "pending_validation"  => ("bg-amber-500/10 text-amber-400 border-amber-500/20 animate-pulse", "Pending DNS"),
            "expired"             => ("bg-red-500/10 text-red-400 border-red-500/20", "Expired"),
            _                     => ("bg-gray-500/10 text-gray-500 border-gray-500/20", "No SSL"),
        }
    };
    view! {
        <span class=format!("inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[10px] font-bold border uppercase tracking-wider {}", bg)>
            <Lock class="w-3 h-3"/>
            {label}
        </span>
    }
}

// ─── Challenge Badge ──────────────────────────────────────────────────────────

#[component]
fn ChallengeBadge(challenge_type: String) -> impl IntoView {
    let (bg, label) = match challenge_type.as_str() {
        "http"             => ("bg-sky-500/10 text-sky-400 border-sky-500/20", "HTTP-01"),
        "dns-digitalocean" => ("bg-blue-500/10 text-blue-400 border-blue-500/20", "DNS · DO"),
        "dns-manual"       => ("bg-amber-500/10 text-amber-400 border-amber-500/20", "DNS · Manual"),
        _                  => ("bg-gray-500/10 text-gray-500 border-gray-500/20", "—"),
    };
    view! {
        <span class=format!("text-[10px] font-bold px-1.5 py-0.5 rounded border {}", bg)>
            {label}
        </span>
    }
}

// ─── Status Badge ─────────────────────────────────────────────────────────────

#[component]
fn ProxyStatusBadge(status: String) -> impl IntoView {
    let (bg, dot, label) = match status.as_str() {
        "active"   => ("bg-green-500/15 border-green-500/30 text-green-400", "bg-green-400", "Active"),
        "inactive" => ("bg-gray-500/15 border-gray-500/30 text-gray-400",   "bg-gray-400",  "Inactive"),
        _          => ("bg-gray-500/15 border-gray-500/30 text-gray-400",   "bg-gray-400",  "Unknown"),
    };
    view! {
        <span class=format!("inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium border {}", bg)>
            <span class=format!("w-1.5 h-1.5 rounded-full {}", dot)></span>
            {label}
        </span>
    }
}

// ─── Stat Card ────────────────────────────────────────────────────────────────

#[component]
fn StatCard(label: &'static str, value: String, color: &'static str) -> impl IntoView {
    let (val_cls, bg_cls) = match color {
        "green"  => ("text-green-400",  "bg-green-500/5 border-green-500/20"),
        "red"    => ("text-red-400",    "bg-red-500/5 border-red-500/20"),
        "blue"   => ("text-blue-400",   "bg-blue-500/5 border-blue-500/20"),
        "purple" => ("text-purple-400", "bg-purple-500/5 border-purple-500/20"),
        _        => ("text-gray-400",   "bg-gray-500/5 border-gray-500/20"),
    };
    view! {
        <div class=format!("rounded-xl border p-5 {}", bg_cls)>
            <p class="text-xs text-gray-500 uppercase tracking-widest font-semibold mb-2">{label}</p>
            <p class=format!("text-3xl font-bold {}", val_cls)>{value}</p>
        </div>
    }
}

// ─── Input helpers ───────────────────────────────────────────────────────────

fn input_cls() -> &'static str {
    "w-full px-3 py-2 bg-gray-800/80 border border-gray-700 rounded-lg \
     text-gray-200 text-sm placeholder-gray-500 \
     focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 transition-all"
}

fn label_cls() -> &'static str {
    "block text-xs font-semibold text-gray-400 uppercase tracking-wide mb-1.5"
}

fn toggle_cls() -> &'static str {
    "w-11 h-6 bg-gray-700 peer-focus:outline-none rounded-full peer \
     peer-checked:after:translate-x-full peer-checked:after:border-white \
     after:content-[''] after:absolute after:top-[2px] after:left-[2px] \
     after:bg-white after:border-gray-300 after:border after:rounded-full \
     after:h-5 after:w-5 after:transition-all peer-checked:bg-emerald-600"
}

// ─── Proxy Page ──────────────────────────────────────────────────────────────

#[component]
pub fn ProxyPage() -> impl IntoView {
    let (show_modal, set_show_modal) = signal(false);
    let (show_creds_modal, set_show_creds_modal) = signal(false);
    let (editing_proxy, set_editing_proxy) = signal(Option::<ProxyEntry>::None);
    let (confirm_delete_id, set_confirm_delete_id) = signal(Option::<String>::None);
    let (show_ssl_modal, set_show_ssl_modal) = signal(Option::<ProxyEntry>::None);
    
    // Form fields for Proxy
    let (f_domain,         set_f_domain)         = signal(String::new());
    let (f_container_id,   set_f_container_id)   = signal(String::new());
    let (f_container_port, set_f_container_port) = signal(80i32);
    let (f_force_https,    set_f_force_https)    = signal(true);
    let (f_auto_ssl,       set_f_auto_ssl)       = signal(true);
    let (f_challenge,      set_f_challenge)      = signal("http".to_string());
    let (f_dns_provider,   set_f_dns_provider)   = signal(String::new());
    let (f_dns_cred_id,    set_f_dns_cred_id)    = signal(String::new());
    
    // Form fields for Credentials
    let (c_name,           set_c_name)           = signal(String::new());
    let (c_provider,       set_c_provider)       = signal("digitalocean".to_string());
    let (c_api_key,        set_c_api_key)        = signal(String::new());
    let (editing_cred,     set_editing_cred)     = signal(Option::<DnsCredential>::None);

    // SSL issue modal fields
    let (ssl_challenge,    set_ssl_challenge)    = signal("http".to_string());
    let (ssl_dns_provider, set_ssl_dns_provider) = signal(String::new());
    let (ssl_dns_cred_id,  set_ssl_dns_cred_id)  = signal(String::new());

    let create_action   = ServerAction::<CreateProxy>::new();
    let update_action   = ServerAction::<UpdateProxy>::new();
    let delete_action   = ServerAction::<DeleteProxy>::new();
    let ssl_action      = ServerAction::<IssueSsl>::new();
    let validate_action = ServerAction::<ValidateDnsChallenge>::new();
    
    let create_cred_action = ServerAction::<CreateDnsCredential>::new();
    let update_cred_action = ServerAction::<UpdateDnsCredential>::new();
    let delete_cred_action = ServerAction::<DeleteDnsCredential>::new();

    let proxies_res = Resource::new(
        move || (
            create_action.version().get(),
            update_action.version().get(),
            delete_action.version().get(),
            ssl_action.version().get(),
            validate_action.version().get(),
        ),
        |_| async { list_proxies().await },
    );

    let creds_res = Resource::new(
        move || (
            create_cred_action.version().get(),
            update_cred_action.version().get(),
            delete_cred_action.version().get(),
        ),
        |_| async { list_dns_credentials().await },
    );

    let containers_res = Resource::new(|| (), |_| async { list_containers().await });

    let user_ctx = expect_context::<crate::app::UserContext>();
    let is_viewer = move || match user_ctx.get() {
        Some(Ok(Some(u))) => u.role == crate::auth::UserRole::Viewer,
        _ => false,
    };

    let do_close = move || {
        set_show_modal.set(false);
        set_editing_proxy.set(None);
        set_f_domain.set(String::new());
        set_f_container_id.set(String::new());
        set_f_container_port.set(80);
        set_f_force_https.set(true);
        set_f_auto_ssl.set(true);
        set_f_challenge.set("http".to_string());
        set_f_dns_provider.set(String::new());
        set_f_dns_cred_id.set(String::new());
    };

    let on_proxy_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if let Some(p) = editing_proxy.get_untracked() {
            update_action.dispatch(UpdateProxy {
                id: p.id,
                domain: f_domain.get_untracked(),
                container_id: f_container_id.get_untracked(),
                container_port: f_container_port.get_untracked(),
                force_https: f_force_https.get_untracked(),
                ssl_challenge_type: f_challenge.get_untracked(),
                dns_provider: f_dns_provider.get_untracked(),
                dns_credential_id: f_dns_cred_id.get_untracked(),
            });
        } else {
            create_action.dispatch(CreateProxy {
                domain: f_domain.get_untracked(),
                container_id: f_container_id.get_untracked(),
                container_port: f_container_port.get_untracked(),
                force_https: f_force_https.get_untracked(),
                auto_ssl: f_auto_ssl.get_untracked(),
                ssl_challenge_type: f_challenge.get_untracked(),
                dns_provider: f_dns_provider.get_untracked(),
                dns_credential_id: f_dns_cred_id.get_untracked(),
            });
        }
        do_close();
    };

    let on_cred_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if let Some(c) = editing_cred.get_untracked() {
            update_cred_action.dispatch(UpdateDnsCredential {
                id: c.id,
                name: c_name.get_untracked(),
                provider: c_provider.get_untracked(),
                api_key: c_api_key.get_untracked(),
            });
        } else {
            create_cred_action.dispatch(CreateDnsCredential {
                name: c_name.get_untracked(),
                provider: c_provider.get_untracked(),
                api_key: c_api_key.get_untracked(),
            });
        }
        set_editing_cred.set(None);
        set_c_name.set(String::new());
        set_c_api_key.set(String::new());
    };

    view! {
        <div class="flex flex-col gap-6">

            // ── Page header ──────────────────────────────────────────────────
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-white tracking-tight flex items-center gap-3">
                        <Globe class="w-8 h-8 text-emerald-500" />
                        "Reverse Proxy"
                    </h1>
                    <p class="text-sm text-gray-500 mt-1">"Route domains to containers with automatic HTTPS via Let's Encrypt"</p>
                </div>
                <div class="flex items-center gap-3">
                    <button
                        on:click=move |_| set_show_creds_modal.set(true)
                        class="flex items-center gap-2 px-4 py-2.5 bg-gray-800 hover:bg-gray-700 text-gray-200 text-sm font-semibold rounded-lg transition-colors border border-gray-700 cursor-pointer"
                    >
                        <Gear class="w-4 h-4"/>
                        "DNS Credentials"
                    </button>
                    <button
                        on:click=move |_| { set_editing_proxy.set(None); set_show_modal.set(true); }
                        disabled=is_viewer
                        class=move || format!("flex items-center gap-2 px-4 py-2.5 bg-emerald-600 hover:bg-emerald-500 \
                               text-white text-sm font-semibold rounded-lg transition-colors shadow-lg \
                               shadow-emerald-500/20 {}", if is_viewer() { "opacity-50 cursor-not-allowed grayscale" } else { "cursor-pointer" })
                    >
                        <Plus class="w-4 h-4"/>
                        "Add Proxy Rule"
                    </button>
                </div>
            </div>

            // ── Stats row ────────────────────────────────────────────────────
            <Suspense fallback=|| view!{
                <div class="grid grid-cols-3 gap-4">
                    {(0..3).map(|_| view!{<div class="h-24 animate-pulse bg-gray-900 rounded-xl border border-gray-800"></div>}).collect_view()}
                </div>
            }>
                {move || proxies_res.get().map(|r| {
                    let proxies = r.unwrap_or_default();
                    let total = proxies.len();
                    let active = proxies.iter().filter(|p| p.status == "active").count();
                    let ssl_secured = proxies.iter().filter(|p| p.ssl_enabled && p.ssl_status == "active").count();
                    view! {
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                            <StatCard label="Total Proxies" value=total.to_string()       color="blue"/>
                            <StatCard label="Active"        value=active.to_string()      color="green"/>
                            <StatCard label="SSL Secured"   value=ssl_secured.to_string() color="purple"/>
                        </div>
                    }.into_any()
                })}
            </Suspense>

            // ── Proxies table ────────────────────────────────────────────────
            <div class="bg-gray-900/60 border border-gray-800 rounded-xl overflow-hidden shadow-xl">
                <Suspense fallback=|| view!{
                    <div class="flex justify-center items-center py-20 text-gray-500 gap-3">
                        <span class="animate-spin text-xl">"⟳"</span>
                        "Loading proxy rules…"
                    </div>
                }>
                    {move || proxies_res.get().map(|r| {
                        let proxies = r.unwrap_or_default();
                        if proxies.is_empty() {
                            view! {
                                <div class="flex flex-col items-center justify-center py-24 gap-4 text-center">
                                    <div class="w-16 h-16 rounded-2xl bg-gray-800 flex items-center justify-center text-3xl">"🌐"</div>
                                    <div>
                                        <p class="text-gray-300 font-semibold">"No proxy rules configured"</p>
                                        <p class="text-gray-600 text-sm mt-1">"Add a proxy rule to route traffic from a domain to a container port"</p>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="overflow-x-auto">
                                    <table class="w-full">
                                        <thead>
                                            <tr class="border-b border-gray-800 bg-gray-900/80">
                                                <th class="px-6 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">"Domain"</th>
                                                <th class="px-6 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">"Target"</th>
                                                <th class="px-6 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">"SSL"</th>
                                                <th class="px-6 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">"Challenge"</th>
                                                <th class="px-6 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">"Status"</th>
                                                <th class="px-6 py-3 text-right text-xs font-semibold text-gray-400 uppercase tracking-wider">"Actions"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-gray-800/60">
                                            {proxies.into_iter().map(|p| {
                                                let p_edit = p.clone();
                                                let p_ssl = p.clone();
                                                let id_del = p.id.clone();
                                                let id_validate = p.id.clone();
                                                let can_issue = !p.ssl_enabled || p.ssl_status == "none" || p.ssl_status == "expired";
                                                let needs_validation = p.ssl_status == "pending_validation";
                                                
                                                view! {
                                                    <tr class="hover:bg-gray-800/30 transition-colors group">
                                                        <td class="px-6 py-4">
                                                            <div class="flex items-center gap-2">
                                                                <Globe class="w-4 h-4 text-emerald-400 flex-shrink-0"/>
                                                                <div class="flex flex-col">
                                                                    <span class="font-semibold text-gray-100">{p.domain.clone()}</span>
                                                                    <span class="text-[10px] text-gray-600 font-mono">{p.id.chars().take(8).collect::<String>()}</span>
                                                                </div>
                                                            </div>
                                                        </td>
                                                        <td class="px-6 py-4">
                                                            <div class="flex flex-col">
                                                                <span class="text-sm text-gray-200 font-medium">{p.container_name.clone()}</span>
                                                                <span class="text-xs text-gray-500 font-mono">
                                                                    {"→ :"}{p.container_port.to_string()}
                                                                </span>
                                                            </div>
                                                        </td>
                                                        <td class="px-6 py-4">
                                                            <SslBadge status=p.ssl_status.clone() enabled=p.ssl_enabled />
                                                        </td>
                                                        <td class="px-6 py-4">
                                                            {if p.ssl_enabled {
                                                                view! { 
                                                                    <div class="flex flex-col gap-1">
                                                                        <ChallengeBadge challenge_type=p.ssl_challenge_type.clone() />
                                                                        {if !p.dns_credential_name.is_empty() {
                                                                            view! { <span class="text-[9px] text-gray-500 truncate max-w-[80px]">{p.dns_credential_name.clone()}</span> }.into_any()
                                                                        } else {
                                                                            view! {}.into_any()
                                                                        }}
                                                                    </div>
                                                                }.into_any()
                                                            } else {
                                                                view! { <span class="text-[10px] text-gray-600 italic">"—"</span> }.into_any()
                                                            }}
                                                        </td>
                                                        <td class="px-6 py-4">
                                                            <ProxyStatusBadge status=p.status.clone()/>
                                                        </td>
                                                        <td class="px-6 py-4 text-right">
                                                            <div class="flex items-center justify-end gap-2">
                                                                // Validate DNS
                                                                {needs_validation.then(|| view! {
                                                                    <button
                                                                        on:click=move |_| { validate_action.dispatch(ValidateDnsChallenge { id: id_validate.clone() }); }
                                                                        disabled=is_viewer
                                                                        class=move || format!("px-2 py-1 text-[10px] font-bold text-amber-400 bg-amber-500/10 border border-amber-500/20 rounded-md hover:bg-amber-500/20 transition-all {}",
                                                                            if is_viewer() { "opacity-30 cursor-not-allowed" } else { "cursor-pointer" })
                                                                    >
                                                                        "✓ Validate"
                                                                    </button>
                                                                })}
                                                                // Issue SSL
                                                                {can_issue.then(|| view! {
                                                                    <button
                                                                        on:click={
                                                                            let p_ssl_clone = p_ssl.clone();
                                                                            move |_| {
                                                                                set_ssl_challenge.set(p_ssl_clone.ssl_challenge_type.clone());
                                                                                set_ssl_dns_provider.set(p_ssl_clone.dns_provider.clone());
                                                                                set_ssl_dns_cred_id.set(p_ssl_clone.dns_credential_id.clone());
                                                                                set_show_ssl_modal.set(Some(p_ssl_clone.clone()));
                                                                            }
                                                                        }
                                                                        disabled=is_viewer
                                                                        class=move || format!("px-2 py-1 text-[10px] font-bold text-emerald-400 bg-emerald-500/10 border border-emerald-500/20 rounded-md hover:bg-emerald-500/20 transition-all {}",
                                                                            if is_viewer() { "opacity-30 cursor-not-allowed" } else { "cursor-pointer" })
                                                                    >
                                                                        <span class="flex items-center gap-1">
                                                                            <Lock class="w-3 h-3"/>
                                                                            "Issue SSL"
                                                                        </span>
                                                                    </button>
                                                                })}
                                                                // Edit
                                                                <button
                                                                    on:click=move |_| {
                                                                        set_f_domain.set(p_edit.domain.clone());
                                                                        set_f_container_id.set(p_edit.container_id.clone());
                                                                        set_f_container_port.set(p_edit.container_port);
                                                                        set_f_force_https.set(p_edit.force_https);
                                                                        set_f_challenge.set(p_edit.ssl_challenge_type.clone());
                                                                        set_f_dns_provider.set(p_edit.dns_provider.clone());
                                                                        set_f_dns_cred_id.set(p_edit.dns_credential_id.clone());
                                                                        set_editing_proxy.set(Some(p_edit.clone()));
                                                                        set_show_modal.set(true);
                                                                    }
                                                                    disabled=is_viewer
                                                                    class=move || format!("p-2 text-gray-400 hover:text-white hover:bg-gray-800 rounded-md transition-colors {}",
                                                                        if is_viewer() { "opacity-30 grayscale cursor-not-allowed" } else { "cursor-pointer" }
                                                                    )
                                                                >"✎"</button>
                                                                // Delete
                                                                <button
                                                                    on:click=move |_| set_confirm_delete_id.set(Some(id_del.clone()))
                                                                    disabled=is_viewer
                                                                    class=move || format!("p-2 text-gray-500 hover:text-red-400 hover:bg-red-400/10 rounded-md transition-colors {}",
                                                                        if is_viewer() { "opacity-30 grayscale cursor-not-allowed" } else { "cursor-pointer" }
                                                                    )
                                                                >
                                                                    <Trash class="w-4 h-4"/>
                                                                </button>
                                                            </div>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            }.into_any()
                        }
                    })}
                </Suspense>
            </div>

            // ── DNS Credentials Modal ──────────────────────────────────────────
            {move || show_creds_modal.get().then(|| view! {
                <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
                    <div class="absolute inset-0 bg-black/70 backdrop-blur-sm" on:click=move |_| set_show_creds_modal.set(false) />
                    <div class="relative bg-gray-900 border border-gray-700 rounded-2xl shadow-2xl w-full max-w-2xl overflow-hidden flex flex-col max-h-[90vh]">
                        <div class="flex items-center justify-between px-6 py-5 border-b border-gray-800">
                            <h2 class="text-base font-bold text-white tracking-tight">"DNS Credentials"</h2>
                            <button on:click=move |_| set_show_creds_modal.set(false) class="text-gray-500 hover:text-white text-2xl transition-colors cursor-pointer">"×"</button>
                        </div>
                        
                        <div class="p-6 overflow-y-auto flex-1 space-y-6">
                            // Form for adding/editing
                            <form on:submit=on_cred_submit class="p-4 bg-gray-950/50 border border-gray-800 rounded-xl space-y-4">
                                <h3 class="text-xs font-bold text-emerald-400 uppercase tracking-widest">{move || if editing_cred.get().is_some() { "Edit Credential" } else { "Add New Credential" }}</h3>
                                <div class="grid grid-cols-2 gap-4">
                                    <div>
                                        <label class=label_cls()>"Name"</label>
                                        <input type="text" required placeholder="e.g. My DigitalOcean Account" class=input_cls() prop:value=c_name on:input=move |ev| set_c_name.set(event_target_value(&ev)) />
                                    </div>
                                    <div>
                                        <label class=label_cls()>"Provider"</label>
                                        <select class=input_cls() on:change=move |ev| set_c_provider.set(event_target_value(&ev))>
                                            <option value="digitalocean" selected=move || c_provider.get() == "digitalocean">"DigitalOcean"</option>
                                        </select>
                                    </div>
                                </div>
                                <div>
                                    <label class=label_cls()>"API Token / Key"</label>
                                    <input type="password" required placeholder="••••••••••••••••" class=input_cls() prop:value=c_api_key on:input=move |ev| set_c_api_key.set(event_target_value(&ev)) />
                                </div>
                                <div class="flex justify-end gap-3">
                                    {move || editing_cred.get().is_some().then(|| view! {
                                        <button type="button" on:click=move |_| { set_editing_cred.set(None); set_c_name.set(String::new()); set_c_api_key.set(String::new()); }
                                            class="px-4 py-2 text-xs text-gray-400 hover:text-white cursor-pointer">"Cancel"</button>
                                    })}
                                    <button type="submit" class="px-5 py-2 bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-bold rounded-lg transition-all cursor-pointer">
                                        {move || if editing_cred.get().is_some() { "Update" } else { "Save Credential" }}
                                    </button>
                                </div>
                            </form>

                            // List of credentials
                            <div class="space-y-3">
                                <h3 class="text-xs font-bold text-gray-500 uppercase tracking-widest">"Active Credentials"</h3>
                                <Suspense fallback=|| view! { <div class="h-20 animate-pulse bg-gray-950/30 rounded-xl border border-gray-800" /> }>
                                    {move || creds_res.get().map(|r| {
                                        let creds = r.unwrap_or_default();
                                        if creds.is_empty() {
                                            view! { <div class="text-center py-6 text-gray-600 text-sm italic">"No credentials configured yet."</div> }.into_any()
                                        } else {
                                            creds.into_iter().map(|c| {
                                                let c_clone = c.clone();
                                                let c_del = c.clone();
                                                view! {
                                                    <div class="flex items-center justify-between p-4 bg-gray-900 border border-gray-800 rounded-xl hover:border-gray-700 transition-colors group">
                                                        <div class="flex items-center gap-3">
                                                            <div class="w-10 h-10 rounded-lg bg-blue-500/10 flex items-center justify-center text-blue-400">
                                                                <Lock class="w-5 h-5"/>
                                                            </div>
                                                            <div>
                                                                <p class="text-sm font-bold text-gray-100">{c.name.clone()}</p>
                                                                <div class="flex items-center gap-2 mt-0.5">
                                                                    <span class="text-[10px] text-blue-400 font-bold uppercase tracking-tighter bg-blue-400/10 px-1 rounded">{c.provider.clone()}</span>
                                                                    <span class="text-[10px] text-gray-600 font-mono italic">{c.api_key.clone()}</span>
                                                                </div>
                                                            </div>
                                                        </div>
                                                        <div class="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                                            <button on:click={
                                                                let c_edit = c_clone.clone();
                                                                move |_| {
                                                                    set_c_name.set(c_edit.name.clone());
                                                                    set_c_provider.set(c_edit.provider.clone());
                                                                    set_c_api_key.set(c_edit.api_key.clone());
                                                                    set_editing_cred.set(Some(c_edit.clone()));
                                                                }
                                                            } class="p-2 text-gray-500 hover:text-white cursor-pointer">"✎"</button>
                                                            <button on:click=move |_| { delete_cred_action.dispatch(DeleteDnsCredential { id: c_del.id.clone() }); } 
                                                                class="p-2 text-gray-500 hover:text-red-400 cursor-pointer"><Trash class="w-4 h-4" /></button>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view().into_any()
                                        }
                                    })}
                                </Suspense>
                            </div>
                        </div>
                    </div>
                </div>
            })}

            // ── Create / Edit Modal ──────────────────────────────────────────
            {move || show_modal.get().then(|| view! {
                <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
                    <div class="absolute inset-0 bg-black/70 backdrop-blur-sm" on:click=move |_| do_close() />
                    <div class="relative bg-gray-900 border border-gray-700 rounded-2xl shadow-2xl w-full max-w-lg overflow-hidden flex flex-col max-h-[90vh]">
                        <div class="flex items-center justify-between px-6 py-5 border-b border-gray-800 flex-shrink-0">
                            <h2 class="text-base font-bold text-white">
                                {move || if editing_proxy.get().is_some() { "Edit Proxy Rule" } else { "Add Proxy Rule" }}
                            </h2>
                            <button on:click=move |_| do_close() class="text-gray-500 hover:text-white text-2xl cursor-pointer transition-colors">"×"</button>
                        </div>

                        <form on:submit=on_proxy_submit class="p-6 space-y-5 overflow-y-auto flex-1">
                            <div>
                                <label class=label_cls()>"Domain (FQDN)"</label>
                                <input type="text" required placeholder="app.example.com" class=input_cls() prop:value=f_domain on:input=move |ev| set_f_domain.set(event_target_value(&ev)) />
                            </div>

                            <div class="grid grid-cols-3 gap-4">
                                <div class="col-span-2">
                                    <label class=label_cls()>"Target Container"</label>
                                    <select class=input_cls() required on:change=move |ev| set_f_container_id.set(event_target_value(&ev))>
                                        <option value="" disabled selected=move || f_container_id.get().is_empty()>"Select container..."</option>
                                        <Suspense fallback=|| view!{ <option disabled>"Loading..."</option> }>
                                            {move || containers_res.get().map(|r| r.unwrap_or_default().into_iter().map(|c| {
                                                view! { <option value=c.id.clone() selected=move || f_container_id.get_untracked() == c.id>{c.name}</option> }
                                            }).collect_view())}
                                        </Suspense>
                                    </select>
                                </div>
                                <div>
                                    <label class=label_cls()>"Port"</label>
                                    <input type="number" min="1" max="65535" required class=input_cls() prop:value=move || f_container_port.get().to_string() on:input=move |ev| { if let Ok(v) = event_target_value(&ev).parse::<i32>() { set_f_container_port.set(v); } } />
                                </div>
                            </div>

                            <div class="p-4 bg-gray-950/50 border border-gray-800 rounded-xl space-y-4">
                                <h3 class="text-xs font-bold text-gray-400 uppercase tracking-widest flex items-center gap-2"><Lock class="w-3.5 h-3.5 text-emerald-400"/>"SSL / HTTPS"</h3>
                                
                                <div class="flex items-center justify-between">
                                    <div>
                                        <p class="text-sm text-gray-200 font-medium">"Force HTTPS"</p>
                                        <p class="text-[10px] text-gray-500">"Redirect HTTP → HTTPS traffic."</p>
                                    </div>
                                    <label class="relative inline-flex items-center cursor-pointer">
                                        <input type="checkbox" class="sr-only peer" prop:checked=move || f_force_https.get() on:change=move |ev| set_f_force_https.set(event_target_checked(&ev)) />
                                        <div class=toggle_cls()></div>
                                    </label>
                                </div>

                                {move || editing_proxy.get().is_none().then(|| view! {
                                    <div class="flex items-center justify-between">
                                        <div><p class="text-sm text-gray-200 font-medium">"Auto-Issue SSL"</p></div>
                                        <label class="relative inline-flex items-center cursor-pointer">
                                            <input type="checkbox" class="sr-only peer" prop:checked=move || f_auto_ssl.get() on:change=move |ev| set_f_auto_ssl.set(event_target_checked(&ev)) />
                                            <div class=toggle_cls()></div>
                                        </label>
                                    </div>
                                })}

                                <div>
                                    <label class=label_cls()>"ACME Challenge Type"</label>
                                    <div class="grid grid-cols-3 gap-2">
                                        {vec![("http", "HTTP-01", "Port 80"), ("dns-digitalocean", "DNS · DO", "API Auto"), ("dns-manual", "DNS · Manual", "TXT Record")].into_iter().map(|(id, title, sub)| {
                                            view! {
                                                <button type="button" on:click=move |_| { set_f_challenge.set(id.into()); if id.starts_with("dns") { set_f_dns_provider.set(if id == "dns-digitalocean" { "digitalocean" } else { "manual" }.into()); } }
                                                    class=move || format!("p-2.5 rounded-lg border text-center transition-all cursor-pointer {}", if f_challenge.get() == id { "border-sky-500 bg-sky-500/10 text-sky-400" } else { "border-gray-700 bg-gray-800/50 text-gray-400 hover:border-gray-600" })>
                                                    <p class="text-xs font-bold">{title}</p><p class="text-[9px] mt-0.5 opacity-60">{sub}</p>
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>

                                {move || match f_challenge.get().as_str() {
                                    "http" => view! { <div class="p-3 bg-sky-500/5 border border-sky-500/10 rounded-lg text-[11px] text-sky-300">"Requires port 80 accessibility for Let's Encrypt validation."</div> }.into_any(),
                                    "dns-digitalocean" => view! {
                                        <div class="space-y-3">
                                            <label class=label_cls()>"Select Credential"</label>
                                            <select class=input_cls() on:change=move |ev| set_f_dns_cred_id.set(event_target_value(&ev))>
                                                <option value="" disabled selected=move || f_dns_cred_id.get().is_empty()>"Select saved credential..."</option>
                                                <Suspense fallback=|| view! { <option disabled>"Loading..."</option> }>
                                                    {move || creds_res.get().map(|r| r.unwrap_or_default().into_iter().filter(|c| c.provider == "digitalocean").map(|c| {
                                                        view! { <option value=c.id.clone() selected=move || f_dns_cred_id.get_untracked() == c.id>{c.name}</option> }
                                                    }).collect_view())}
                                                </Suspense>
                                            </select>
                                            <div class="flex justify-between items-center px-1">
                                                <p class="text-[10px] text-gray-500">"Automated DNS-01 via DigitalOcean API."</p>
                                                <button type="button" on:click=move |_| set_show_creds_modal.set(true) class="text-[10px] text-blue-400 hover:underline cursor-pointer">"+ Add New"</button>
                                            </div>
                                        </div>
                                    }.into_any(),
                                    "dns-manual" => view! { <div class="p-3 bg-amber-500/5 border border-amber-500/10 rounded-lg text-[11px] text-amber-300">"Manual TXT record addition required. Instructions provided after creation."</div> }.into_any(),
                                    _ => view! { <div></div> }.into_any(),
                                }}
                            </div>

                            <div class="flex justify-end gap-3 pt-4 border-t border-gray-800">
                                <button type="button" on:click=move |_| do_close() class="px-4 py-2 text-sm text-gray-400 hover:text-white cursor-pointer transition-colors">"Cancel"</button>
                                <button type="submit" class="px-6 py-2 bg-emerald-600 hover:bg-emerald-500 text-white text-sm font-semibold rounded-lg transition-all shadow-lg shadow-emerald-500/20 active:scale-95 cursor-pointer">
                                    {move || if editing_proxy.get().is_some() { "Save Changes" } else { "Create Proxy" }}
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            })}

            // ── Issue SSL Modal ───────────────────────────────────────────────
            {move || show_ssl_modal.get().map(|proxy| {
                let proxy_id = proxy.id.clone();
                
                view! {
                    <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
                        <div class="absolute inset-0 bg-black/70 backdrop-blur-sm" on:click=move |_| set_show_ssl_modal.set(None) />
                        <div class="relative bg-gray-900 border border-gray-700 rounded-2xl shadow-2xl w-full max-w-lg overflow-hidden flex flex-col max-h-[90vh]">
                            <div class="flex items-center justify-between px-6 py-5 border-b border-gray-800">
                                <div><h2 class="text-base font-bold text-white">"Issue SSL Certificate"</h2><p class="text-[11px] text-gray-500 font-mono italic mt-0.5">{proxy.domain.clone()}</p></div>
                                <button on:click=move |_| set_show_ssl_modal.set(None) class="text-gray-500 hover:text-white text-2xl transition-colors cursor-pointer">"×"</button>
                            </div>

                            <div class="p-6 space-y-6 overflow-y-auto flex-1">
                                <div>
                                    <label class=label_cls()>"Challenge Strategy"</label>
                                    <div class="grid grid-cols-3 gap-2">
                                        {vec![("http", "HTTP-01"), ("dns-digitalocean", "DNS · DO"), ("dns-manual", "DNS · Manual")].into_iter().map(|(id, title)| {
                                            view! {
                                                <button type="button" on:click=move |_| { set_ssl_challenge.set(id.into()); if id.starts_with("dns") { set_ssl_dns_provider.set(if id == "dns-digitalocean" { "digitalocean" } else { "manual" }.into()); } }
                                                    class=move || format!("p-2.5 rounded-lg border text-center transition-all cursor-pointer {}", if ssl_challenge.get() == id { "border-sky-500 bg-sky-500/10 text-sky-400" } else { "border-gray-700 bg-gray-800/50 text-gray-400 hover:border-gray-600" })>
                                                    <p class="text-xs font-bold">{title}</p>
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>

                                {move || match ssl_challenge.get().as_str() {
                                    "dns-digitalocean" => view! {
                                        <div class="space-y-3">
                                            <label class=label_cls()>"Saved Credential"</label>
                                            <select class=input_cls() on:change=move |ev| set_ssl_dns_cred_id.set(event_target_value(&ev))>
                                                <option value="" disabled selected=move || ssl_dns_cred_id.get().is_empty()>"Select credential..."</option>
                                                <Suspense fallback=|| view! { <option disabled>"Loading..."</option> }>
                                                    {move || creds_res.get().map(|r| r.unwrap_or_default().into_iter().filter(|c| c.provider == "digitalocean").map(|c| {
                                                        view! { <option value=c.id.clone() selected=move || ssl_dns_cred_id.get_untracked() == c.id>{c.name}</option> }
                                                    }).collect_view())}
                                                </Suspense>
                                            </select>
                                        </div>
                                    }.into_any(),
                                    _ => view! {}.into_any(),
                                }}

                                <div class="flex justify-end gap-3 pt-6 border-t border-gray-800">
                                    <button on:click=move |_| set_show_ssl_modal.set(None) class="px-4 py-2 text-sm text-gray-500 hover:text-white transition-colors cursor-pointer">"Cancel"</button>
                                    <button on:click=move |_| {
                                        ssl_action.dispatch(IssueSsl { id: proxy_id.clone(), challenge_type: ssl_challenge.get_untracked(), dns_provider: ssl_dns_provider.get_untracked(), dns_credential_id: ssl_dns_cred_id.get_untracked() });
                                        set_show_ssl_modal.set(None);
                                    } class="px-6 py-2 bg-emerald-600 hover:bg-emerald-500 text-white text-sm font-bold rounded-lg transition-all shadow-lg shadow-emerald-500/20 active:scale-95 cursor-pointer">
                                        "Renew Certificate"
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                }
            })}

            // ── Delete Confirm ────────────────────────────────────────────────
            {move || confirm_delete_id.get().map(|del_id| {
                let id_clone = del_id.clone();
                view! {
                    <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
                        <div class="absolute inset-0 bg-black/70 backdrop-blur-sm" on:click=move |_| set_confirm_delete_id.set(None) />
                        <div class="relative bg-gray-900 border border-red-900/40 rounded-2xl shadow-2xl p-6 max-w-sm w-full">
                            <div class="flex items-center gap-3 mb-3"><span class="bg-red-500/20 p-2 rounded-lg text-red-500 text-xl">"⚠"</span><h2 class="text-base font-bold text-white">"Delete Proxy Rule"</h2></div>
                            <p class="text-gray-400 text-sm mb-6 leading-relaxed">"This will remove the proxy configuration and revoke any associated SSL certificate."</p>
                            <div class="flex gap-3 justify-end">
                                <button on:click=move |_| set_confirm_delete_id.set(None) class="px-4 py-2 text-sm text-gray-400 hover:text-white cursor-pointer">"Cancel"</button>
                                <button on:click=move |_| { delete_action.dispatch(DeleteProxy { id: id_clone.clone() }); set_confirm_delete_id.set(None); } class="px-4 py-2 bg-red-600 hover:bg-red-500 text-white text-sm font-bold rounded-lg transition-colors cursor-pointer">"Delete"</button>
                            </div>
                        </div>
                    </div>
                }
            })}
        </div>
    }
}

fn event_target_checked(ev: &leptos::ev::Event) -> bool {
    use leptos::wasm_bindgen::JsCast;
    ev.target()
        .and_then(|t| t.dyn_into::<leptos::web_sys::HtmlInputElement>().ok())
        .map(|el: leptos::web_sys::HtmlInputElement| el.checked())
        .unwrap_or(false)
}
