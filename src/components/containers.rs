use leptos::prelude::*;
use crate::containers::*;
use crate::hosts::list_hosts;
use crate::components::icons::Stack;

// ─── Status Badge ─────────────────────────────────────────────────────────────

#[component]
fn StatusBadge(status: String) -> impl IntoView {
    let (bg, dot, label) = match status.as_str() {
        "running" => ("bg-green-500/15 border-green-500/30 text-green-400", "bg-green-400", "Running"),
        "stopped" => ("bg-red-500/15 border-red-500/30 text-red-400", "bg-red-400", "Stopped"),
        "restarting" => ("bg-yellow-500/15 border-yellow-500/30 text-yellow-400", "bg-yellow-400", "Restarting"),
        _         => ("bg-gray-500/15 border-gray-500/30 text-gray-400", "bg-gray-400", "Unknown"),
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
        "green" => ("text-green-400", "bg-green-500/5 border-green-500/20"),
        "red"   => ("text-red-400",   "bg-red-500/5 border-red-500/20"),
        "blue"  => ("text-blue-400",  "bg-blue-500/5 border-blue-500/20"),
        _       => ("text-gray-400",  "bg-gray-500/5 border-gray-500/20"),
    };
    view! {
        <div class=format!("rounded-xl border p-5 {}", bg_cls)>
            <p class="text-xs text-gray-500 uppercase tracking-widest font-semibold mb-2">{label}</p>
            <p class=format!("text-3xl font-bold {}", val_cls)>{value}</p>
        </div>
    }
}

// ─── Input helper ─────────────────────────────────────────────────────────────

fn input_cls() -> &'static str {
    "w-full px-3 py-2 bg-gray-800/80 border border-gray-700 rounded-lg \
     text-gray-200 text-sm placeholder-gray-500 \
     focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 transition-all"
}

fn label_cls() -> &'static str {
    "block text-xs font-semibold text-gray-400 uppercase tracking-wide mb-1.5"
}

fn small_input_cls() -> &'static str {
    "w-full px-2 py-1.5 bg-gray-800/80 border border-gray-700 rounded-md \
     text-gray-200 text-xs placeholder-gray-500 \
     focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 transition-all"
}

// ─── Containers Page ─────────────────────────────────────────────────────────

#[component]
pub fn ContainersPage() -> impl IntoView {
    // Modal / form state
    let (show_modal, set_show_modal) = signal(false);
    let (editing_container, set_editing_container) = signal(Option::<Container>::None);
    let (confirm_delete_id, set_confirm_delete_id) = signal(Option::<String>::None);
    
    let (f_name,      set_f_name)      = signal(String::new());
    let (f_host_id,   set_f_host_id)   = signal(String::new());
    let (f_image,     set_f_image)     = signal(String::new());
    let (f_cpu,       set_f_cpu)       = signal(0.5f64);
    let (f_memory,    set_f_memory)    = signal(512i32);
    
    // Dynamic config lists
    let (f_env_vars, set_f_env_vars)  = signal(Vec::<(String, String)>::new());
    let (f_volumes, set_f_volumes)    = signal(Vec::<(String, String)>::new());
    let (f_ports, set_f_ports)        = signal(Vec::<(String, String, String)>::new());

    // Server actions
    let create_action = ServerAction::<CreateContainer>::new();
    let update_action = ServerAction::<UpdateContainer>::new();
    let delete_action = ServerAction::<DeleteContainer>::new();
    let status_action = ServerAction::<ToggleContainerStatus>::new();

    // Resource — refetches whenever any action version changes
    let containers_res = Resource::new(
        move || (
            create_action.version().get(),
            update_action.version().get(),
            delete_action.version().get(),
            status_action.version().get(),
        ),
        |_| async { list_containers().await },
    );

    // Fetch hosts for selection
    let hosts_res = Resource::new(|| (), |_| async { list_hosts().await });

    let user_ctx = expect_context::<crate::app::UserContext>();
    let is_viewer = move || match user_ctx.get() {
        Some(Ok(Some(u))) => u.role == crate::auth::UserRole::Viewer,
        _ => false,
    };

    // Reset / close modal
    let do_close = move || {
        set_show_modal.set(false);
        set_editing_container.set(None);
        set_f_name.set(String::new());
        set_f_host_id.set(String::new());
        set_f_image.set(String::new());
        set_f_cpu.set(0.5);
        set_f_memory.set(512);
        set_f_env_vars.set(Vec::new());
        set_f_volumes.set(Vec::new());
        set_f_ports.set(Vec::new());
    };

    // Form submit
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        
        // Serialize dynamic fields to JSON strings
        let env_json = {
            let items: Vec<EnvVar> = f_env_vars.get_untracked().into_iter()
                .filter(|(k, _)| !k.is_empty())
                .map(|(k, v)| EnvVar { key: k, value: v })
                .collect();
            serde_json::to_string(&items).unwrap_or_else(|_| "[]".into())
        };
        let vol_json = {
            let items: Vec<Volume> = f_volumes.get_untracked().into_iter()
                .filter(|(h, _)| !h.is_empty())
                .map(|(h, c)| Volume { host_path: h, container_path: c })
                .collect();
            serde_json::to_string(&items).unwrap_or_else(|_| "[]".into())
        };
        let port_json = {
            let items: Vec<PortMapping> = f_ports.get_untracked().into_iter()
                .filter(|(h, _, _)| !h.is_empty())
                .map(|(h, c, p)| PortMapping { 
                    host_port: h.parse().unwrap_or(0), 
                    container_port: c.parse().unwrap_or(0), 
                    protocol: if p.is_empty() { "tcp".into() } else { p }
                })
                .collect();
            serde_json::to_string(&items).unwrap_or_else(|_| "[]".into())
        };
        
        if let Some(c) = editing_container.get_untracked() {
            update_action.dispatch(UpdateContainer {
                id: c.id,
                host_id: f_host_id.get_untracked(),
                name: f_name.get_untracked(),
                image: f_image.get_untracked(),
                cpu_limit: f_cpu.get_untracked(),
                memory_limit_mb: f_memory.get_untracked(),
                env_vars: env_json,
                volumes: vol_json,
                ports: port_json,
            });
        } else {
            create_action.dispatch(CreateContainer {
                host_id: f_host_id.get_untracked(),
                name: f_name.get_untracked(),
                image: f_image.get_untracked(),
                cpu_limit: f_cpu.get_untracked(),
                memory_limit_mb: f_memory.get_untracked(),
                env_vars: env_json,
                volumes: vol_json,
                ports: port_json,
            });
        }
        do_close();
    };

    view! {
        <div class="flex flex-col gap-6">

            // ── Page header ──────────────────────────────────────────────────
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-white tracking-tight flex items-center gap-3">
                        <Stack class="w-8 h-8 text-blue-500" />
                        "Containers"
                    </h1>
                    <p class="text-sm text-gray-500 mt-1">"Deploy and manage containerized workloads" </p>
                </div>
                <button
                    on:click=move |_| { set_editing_container.set(None); set_show_modal.set(true); }
                    disabled=is_viewer
                    class=move || format!("flex items-center gap-2 px-4 py-2.5 bg-blue-600 hover:bg-blue-500 \
                           text-white text-sm font-semibold rounded-lg transition-colors shadow-lg \
                           shadow-blue-500/20 {}", if is_viewer() { "opacity-50 cursor-not-allowed grayscale" } else { "cursor-pointer" })
                >
                    <span class="text-base leading-none">"+"</span>
                    "Deploy Container"
                </button>
            </div>

            // ── Stats row ────────────────────────────────────────────────────
            <Suspense fallback=|| view!{
                <div class="grid grid-cols-3 gap-4">
                    {(0..3).map(|_| view!{<div class="h-24 animate-pulse bg-gray-900 rounded-xl border border-gray-800"></div>}).collect_view()}
                </div>
            }>
                {move || containers_res.get().map(|r| {
                    let containers = r.unwrap_or_default();
                    let total   = containers.len();
                    let running = containers.iter().filter(|c| c.status == "running").count();
                    let stopped = containers.iter().filter(|c| c.status == "stopped").count();
                    view! {
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                            <StatCard label="Total Containers" value=total.to_string()   color="blue"/>
                            <StatCard label="Running"          value=running.to_string() color="green"/>
                            <StatCard label="Stopped"          value=stopped.to_string() color="red"/>
                        </div>
                    }.into_any()
                })}
            </Suspense>

            // ── Containers table ─────────────────────────────────────────────
            <div class="bg-gray-900/60 border border-gray-800 rounded-xl overflow-hidden shadow-xl">
                <Suspense fallback=|| view!{
                    <div class="flex justify-center items-center py-20 text-gray-500 gap-3">
                        <span class="animate-spin text-xl">"⟳"</span>
                        "Loading containers…"
                    </div>
                }>
                    {move || containers_res.get().map(|r| {
                        let containers = r.unwrap_or_default();
                        if containers.is_empty() {
                            view! {
                                <div class="flex flex-col items-center justify-center py-24 gap-4 text-center">
                                    <div class="w-16 h-16 rounded-2xl bg-gray-800 flex items-center justify-center text-3xl">"📦"</div>
                                    <div>
                                        <p class="text-gray-300 font-semibold">"No containers deployed"</p>
                                        <p class="text-gray-600 text-sm mt-1">"Click \"Deploy Container\" to start a new service"</p>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="overflow-x-auto">
                                    <table class="w-full">
                                        <thead>
                                            <tr class="border-b border-gray-800 bg-gray-900/80">
                                                <th class="px-6 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">"Name"</th>
                                                <th class="px-6 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">"Image"</th>
                                                <th class="px-6 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">"Host"</th>
                                                <th class="px-6 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">"Config"</th>
                                                <th class="px-6 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">"Status"</th>
                                                <th class="px-6 py-3 text-right text-xs font-semibold text-gray-400 uppercase tracking-wider">"Actions"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-gray-800/60">
                                            {containers.into_iter().map(|c| {
                                                let c_clone = c.clone();
                                                let id_st  = c.id.clone();
                                                let id_del = c.id.clone();
                                                let is_running = c.status == "running";
                                                let port_count = c.ports.len();
                                                let vol_count = c.volumes.len();
                                                let env_count = c.env_vars.len();
                                                let ports_display: Vec<String> = c.ports.iter().take(3).map(|p| format!("{}:{}", p.host_port, p.container_port)).collect();
                                                
                                                view! {
                                                    <tr class="hover:bg-gray-800/30 transition-colors">
                                                        <td class="px-6 py-4">
                                                            <div class="flex flex-col">
                                                                <span class="font-semibold text-gray-100">{c.name.clone()}</span>
                                                                <span class="text-[10px] text-gray-600 font-mono tracking-tighter truncate max-w-[100px]">{c.id.clone()}</span>
                                                            </div>
                                                        </td>
                                                        <td class="px-6 py-4 text-sm text-gray-400 font-mono">
                                                            {c.image.clone()}
                                                        </td>
                                                        <td class="px-6 py-4 text-sm text-gray-400">
                                                            {c.host_name.clone()}
                                                        </td>
                                                        <td class="px-6 py-4">
                                                            <div class="flex flex-wrap gap-1.5">
                                                                {(port_count > 0).then(|| view! {
                                                                    <span class="text-[10px] font-bold px-1.5 py-0.5 rounded bg-blue-500/10 text-blue-400 border border-blue-500/20" title=ports_display.join(", ")>
                                                                        {format!("{} port{}", port_count, if port_count == 1 { "" } else { "s" })}
                                                                    </span>
                                                                })}
                                                                {(vol_count > 0).then(|| view! {
                                                                    <span class="text-[10px] font-bold px-1.5 py-0.5 rounded bg-purple-500/10 text-purple-400 border border-purple-500/20">
                                                                        {format!("{} vol{}", vol_count, if vol_count == 1 { "" } else { "s" })}
                                                                    </span>
                                                                })}
                                                                {(env_count > 0).then(|| view! {
                                                                    <span class="text-[10px] font-bold px-1.5 py-0.5 rounded bg-green-500/10 text-green-400 border border-green-500/20">
                                                                        {format!("{} env", env_count)}
                                                                    </span>
                                                                })}
                                                                {(port_count == 0 && vol_count == 0 && env_count == 0).then(|| view! {
                                                                    <span class="text-[10px] text-gray-600 italic">"—"</span>
                                                                })}
                                                            </div>
                                                        </td>
                                                        <td class="px-6 py-4">
                                                            <StatusBadge status=c.status.clone()/>
                                                        </td>
                                                        <td class="px-6 py-4 text-right">
                                                             <div class="flex items-center justify-end gap-2">
                                                                // Toggle Status
                                                                <button
                                                                    on:click=move |_| { status_action.dispatch(ToggleContainerStatus { id: id_st.clone() }); }
                                                                    disabled=is_viewer
                                                                    class=move || format!("p-2 rounded-md transition-colors {} {}", 
                                                                        if is_running { "text-red-400 hover:bg-red-400/10" } else { "text-green-400 hover:bg-green-400/10" },
                                                                        if is_viewer() { "opacity-30 grayscale cursor-not-allowed" } else { "cursor-pointer" }
                                                                    )
                                                                    title=if is_running { "Stop" } else { "Start" }
                                                                >
                                                                    {if is_running { "■" } else { "▶" }}
                                                                </button>
                                                                
                                                                // Edit
                                                                <button
                                                                    on:click=move |_| {
                                                                        set_f_name.set(c_clone.name.clone());
                                                                        set_f_host_id.set(c_clone.host_id.clone());
                                                                        set_f_image.set(c_clone.image.clone());
                                                                        set_f_cpu.set(c_clone.cpu_limit);
                                                                        set_f_memory.set(c_clone.memory_limit_mb);
                                                                        set_f_env_vars.set(c_clone.env_vars.iter().map(|e| (e.key.clone(), e.value.clone())).collect());
                                                                        set_f_volumes.set(c_clone.volumes.iter().map(|v| (v.host_path.clone(), v.container_path.clone())).collect());
                                                                        set_f_ports.set(c_clone.ports.iter().map(|p| (p.host_port.to_string(), p.container_port.to_string(), p.protocol.clone())).collect());
                                                                        set_editing_container.set(Some(c_clone.clone()));
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
                                                                >"🗑"</button>
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

            // ── Create / Edit Modal ──────────────────────────────────────────
            {move || show_modal.get().then(|| view! {
                <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
                    <div
                        class="absolute inset-0 bg-black/70 backdrop-blur-sm"
                        on:click=move |_| do_close()
                    />
                    <div class="relative bg-gray-900 border border-gray-700 rounded-2xl shadow-2xl w-full max-w-2xl overflow-hidden flex flex-col max-h-[90vh]">
                        <div class="flex items-center justify-between px-6 py-5 border-b border-gray-800 flex-shrink-0">
                            <h2 class="text-base font-bold text-white">
                                {move || if editing_container.get().is_some() { "Edit Container" } else { "Deploy Container" }}
                            </h2>
                            <button on:click=move |_| do_close()
                                class="text-gray-500 hover:text-white text-2xl leading-none transition-colors">"×"</button>
                        </div>

                        <form on:submit=on_submit class="p-6 space-y-5 overflow-y-auto flex-1">
                            // Name
                            <div>
                                <label class=label_cls()>"Container Name"</label>
                                <input type="text" required placeholder="e.g. nginx-lb-01"
                                    class=input_cls()
                                    prop:value=f_name
                                    on:input=move |ev| set_f_name.set(event_target_value(&ev))
                                />
                            </div>

                            // Host Selection
                            <div>
                                <label class=label_cls()>"Target Host"</label>
                                <select class=input_cls() required
                                    on:change=move |ev| set_f_host_id.set(event_target_value(&ev))
                                >
                                    <option value="" disabled selected=move || f_host_id.get().is_empty()>"Select target host..."</option>
                                    <Suspense fallback=|| view!{ <option disabled>"Loading hosts..."</option> }>
                                        {move || hosts_res.get().map(|r| {
                                            r.unwrap_or_default().into_iter().map(|host| {
                                                let selected = f_host_id.get_untracked() == host.id;
                                                view! { <option value=host.id.clone() selected=selected>{host.name}</option> }
                                            }).collect_view()
                                        })}
                                    </Suspense>
                                </select>
                            </div>

                            // Image
                            <div>
                                <label class=label_cls()>"Container Image"</label>
                                <input type="text" required placeholder="e.g. nginx:latest, postgres:15-alpine"
                                    class=input_cls()
                                    prop:value=f_image
                                    on:input=move |ev| set_f_image.set(event_target_value(&ev))
                                />
                            </div>

                            // Resources
                            <div class="grid grid-cols-2 gap-4">
                                <div>
                                    <label class=label_cls()>"CPU Limit"</label>
                                    <input type="number" min="0.1" step="0.1" required
                                        class=input_cls()
                                        prop:value=move || f_cpu.get().to_string()
                                        on:input=move |ev| {
                                            if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                                                set_f_cpu.set(val);
                                            }
                                        }
                                    />
                                </div>
                                <div>
                                    <label class=label_cls()>"Memory Limit (MB)"</label>
                                    <input type="number" min="64" step="64" required
                                        class=input_cls()
                                        prop:value=move || f_memory.get().to_string()
                                        on:input=move |ev| {
                                            if let Ok(val) = event_target_value(&ev).parse::<i32>() {
                                                set_f_memory.set(val);
                                            }
                                        }
                                    />
                                </div>
                            </div>

                            // ── Port Mappings ────────────────────────────────
                            <div class="space-y-2">
                                <div class="flex items-center justify-between">
                                    <label class=label_cls()>"Port Mappings"</label>
                                    <button type="button"
                                        on:click=move |_| set_f_ports.update(|v| v.push(("".into(), "".into(), "tcp".into())))
                                        class="text-[10px] font-bold text-blue-400 hover:text-blue-300 px-2 py-0.5 bg-blue-500/10 rounded border border-blue-500/20 transition cursor-pointer"
                                    >"+ Port"</button>
                                </div>
                                {move || {
                                    let items = f_ports.get();
                                    if items.is_empty() {
                                        view! { <p class="text-xs text-gray-600 italic">"No port mappings configured."</p> }.into_any()
                                    } else {
                                        items.into_iter().enumerate().map(|(i, (hp, cp, proto))| {
                                            view! {
                                                <div class="flex items-center gap-2">
                                                    <input type="number" min="1" max="65535" placeholder="Host"
                                                        class=small_input_cls()
                                                        prop:value=hp.clone()
                                                        on:input=move |ev| set_f_ports.update(|v| { if let Some(item) = v.get_mut(i) { item.0 = event_target_value(&ev); } })
                                                    />
                                                    <span class="text-gray-600 text-xs font-mono">"→"</span>
                                                    <input type="number" min="1" max="65535" placeholder="Container"
                                                        class=small_input_cls()
                                                        prop:value=cp.clone()
                                                        on:input=move |ev| set_f_ports.update(|v| { if let Some(item) = v.get_mut(i) { item.1 = event_target_value(&ev); } })
                                                    />
                                                    <select class="px-2 py-1.5 bg-gray-800/80 border border-gray-700 rounded-md text-gray-200 text-xs focus:outline-none focus:ring-1 focus:ring-blue-500 transition-all"
                                                        on:change=move |ev| set_f_ports.update(|v| { if let Some(item) = v.get_mut(i) { item.2 = event_target_value(&ev); } })
                                                    >
                                                        <option value="tcp" selected=proto == "tcp">"TCP"</option>
                                                        <option value="udp" selected=proto == "udp">"UDP"</option>
                                                    </select>
                                                    <button type="button"
                                                        on:click=move |_| set_f_ports.update(|v| { v.remove(i); })
                                                        class="p-1 text-red-400 hover:text-red-300 hover:bg-red-400/10 rounded transition cursor-pointer"
                                                    >"✕"</button>
                                                </div>
                                            }
                                        }).collect_view().into_any()
                                    }
                                }}
                            </div>

                            // ── Volumes ──────────────────────────────────────
                            <div class="space-y-2">
                                <div class="flex items-center justify-between">
                                    <label class=label_cls()>"Volumes"</label>
                                    <button type="button"
                                        on:click=move |_| set_f_volumes.update(|v| v.push(("".into(), "".into())))
                                        class="text-[10px] font-bold text-purple-400 hover:text-purple-300 px-2 py-0.5 bg-purple-500/10 rounded border border-purple-500/20 transition cursor-pointer"
                                    >"+ Volume"</button>
                                </div>
                                {move || {
                                    let items = f_volumes.get();
                                    if items.is_empty() {
                                        view! { <p class="text-xs text-gray-600 italic">"No volumes mounted."</p> }.into_any()
                                    } else {
                                        items.into_iter().enumerate().map(|(i, (hp, cp))| {
                                            view! {
                                                <div class="flex items-center gap-2">
                                                    <input type="text" placeholder="/host/path"
                                                        class=small_input_cls()
                                                        prop:value=hp.clone()
                                                        on:input=move |ev| set_f_volumes.update(|v| { if let Some(item) = v.get_mut(i) { item.0 = event_target_value(&ev); } })
                                                    />
                                                    <span class="text-gray-600 text-xs font-mono">"→"</span>
                                                    <input type="text" placeholder="/container/path"
                                                        class=small_input_cls()
                                                        prop:value=cp.clone()
                                                        on:input=move |ev| set_f_volumes.update(|v| { if let Some(item) = v.get_mut(i) { item.1 = event_target_value(&ev); } })
                                                    />
                                                    <button type="button"
                                                        on:click=move |_| set_f_volumes.update(|v| { v.remove(i); })
                                                        class="p-1 text-red-400 hover:text-red-300 hover:bg-red-400/10 rounded transition cursor-pointer"
                                                    >"✕"</button>
                                                </div>
                                            }
                                        }).collect_view().into_any()
                                    }
                                }}
                            </div>

                            // ── Environment Variables ─────────────────────────
                            <div class="space-y-2">
                                <div class="flex items-center justify-between">
                                    <label class=label_cls()>"Environment Variables"</label>
                                    <button type="button"
                                        on:click=move |_| set_f_env_vars.update(|v| v.push(("".into(), "".into())))
                                        class="text-[10px] font-bold text-green-400 hover:text-green-300 px-2 py-0.5 bg-green-500/10 rounded border border-green-500/20 transition cursor-pointer"
                                    >"+ Env Var"</button>
                                </div>
                                {move || {
                                    let items = f_env_vars.get();
                                    if items.is_empty() {
                                        view! { <p class="text-xs text-gray-600 italic">"No environment variables defined."</p> }.into_any()
                                    } else {
                                        items.into_iter().enumerate().map(|(i, (key, val))| {
                                            view! {
                                                <div class="flex items-center gap-2">
                                                    <input type="text" placeholder="KEY"
                                                        class=small_input_cls()
                                                        prop:value=key.clone()
                                                        on:input=move |ev| set_f_env_vars.update(|v| { if let Some(item) = v.get_mut(i) { item.0 = event_target_value(&ev); } })
                                                    />
                                                    <span class="text-gray-600 text-xs font-mono">"="</span>
                                                    <input type="text" placeholder="value"
                                                        class=small_input_cls()
                                                        prop:value=val.clone()
                                                        on:input=move |ev| set_f_env_vars.update(|v| { if let Some(item) = v.get_mut(i) { item.1 = event_target_value(&ev); } })
                                                    />
                                                    <button type="button"
                                                        on:click=move |_| set_f_env_vars.update(|v| { v.remove(i); })
                                                        class="p-1 text-red-400 hover:text-red-300 hover:bg-red-400/10 rounded transition cursor-pointer"
                                                    >"✕"</button>
                                                </div>
                                            }
                                        }).collect_view().into_any()
                                    }
                                }}
                            </div>

                            <div class="flex justify-end gap-3 pt-6 border-t border-gray-800 mt-2">
                                <button type="button" on:click=move |_| do_close()
                                    class="px-4 py-2 text-sm text-gray-400 hover:text-white transition-colors">
                                    "Cancel"
                                </button>
                                <button type="submit"
                                    class="px-6 py-2 bg-blue-600 hover:bg-blue-500 text-white \
                                           text-sm font-semibold rounded-lg transition-all shadow-lg \
                                           shadow-blue-500/20 active:scale-95">
                                    {move || if editing_container.get().is_some() { "Save Changes" } else { "Deploy" }}
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            })}

            // ── Delete Confirm Modal ─────────────────────────────────────────
            {move || confirm_delete_id.get().map(|del_id| {
                let id_clone = del_id.clone();
                view! {
                    <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
                        <div class="absolute inset-0 bg-black/70 backdrop-blur-sm"/>
                        <div class="relative bg-gray-900 border border-red-900/40 rounded-2xl shadow-2xl p-6 max-w-sm w-full">
                            <div class="flex items-center gap-3 mb-3">
                                <span class="bg-red-500/20 p-2 rounded-lg text-red-500 text-xl">"⚠"</span>
                                <h2 class="text-base font-bold text-white">"Delete Container"</h2>
                            </div>
                            <p class="text-gray-400 text-sm mb-6 leading-relaxed">
                                "Are you sure you want to delete this container? All ephemeral data and configuration will be permanently purged."
                            </p>
                            <div class="flex gap-3 justify-end">
                                <button on:click=move |_| set_confirm_delete_id.set(None)
                                    class="px-4 py-2 text-sm text-gray-400 hover:text-white transition-colors">
                                    "Cancel"
                                </button>
                                <button on:click=move |_| {
                                    delete_action.dispatch(DeleteContainer { id: id_clone.clone() });
                                    set_confirm_delete_id.set(None);
                                }
                                    class="px-4 py-2 bg-red-600 hover:bg-red-500 text-white \
                                           text-sm font-semibold rounded-lg transition-colors">
                                    "Delete"
                                </button>
                            </div>
                        </div>
                    </div>
                }
            })}
        </div>
    }
}
