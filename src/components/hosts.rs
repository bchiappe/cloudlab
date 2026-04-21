use leptos::prelude::*;
use crate::hosts::*;
use crate::llms::DeployFox;

// ─── Status Badge ─────────────────────────────────────────────────────────────

#[component]
fn StatusBadge(status: String) -> impl IntoView {
    let (bg, dot, label) = match status.as_str() {
        "online"  => ("bg-green-500/15 border-green-500/30 text-green-400",  "bg-green-400",  "Online"),
        "offline" => ("bg-red-500/15 border-red-500/30 text-red-400",        "bg-red-400",    "Offline"),
        _         => ("bg-gray-500/15 border-gray-500/30 text-gray-400",     "bg-gray-400",   "Unknown"),
    };
    view! {
        <span class=format!("inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium border {}", bg)>
            <span class=format!("w-1.5 h-1.5 rounded-full {}", dot)></span>
            {label}
        </span>
    }
}

// ─── Resource Bar ─────────────────────────────────────────────────────────────

#[component]
fn ResourceBar(label: &'static str, value: f64, color: &'static str) -> impl IntoView {
    let bar_cls = match color {
        "blue"  => "bg-blue-500",
        "green" => "bg-green-500",
        "red"   => "bg-red-500",
        _       => "bg-gray-500",
    };
    view! {
        <div class="flex flex-col gap-1 w-full max-w-[100px]">
            <div class="flex justify-between items-center text-[10px] font-bold text-gray-400 uppercase tracking-tighter">
                <span>{label}</span>
                <span>{format!("{:.0}%", value)}</span>
            </div>
            <div class="h-1.5 w-full bg-gray-800 rounded-full overflow-hidden">
                <div 
                    class=format!("h-full transition-all duration-1000 {}", bar_cls) 
                    style=format!("width: {}%", value.min(100.0))
                />
            </div>
        </div>
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

// ─── Hosts Page ───────────────────────────────────────────────────────────────

#[component]
pub fn HostsPage() -> impl IntoView {
    // Modal / form state
    let (show_modal, set_show_modal) = signal(false);
    let (editing_host, set_editing_host) = signal(Option::<Host>::None);
    let (confirm_delete_id, set_confirm_delete_id) = signal(Option::<String>::None);
    let (testing_id, set_testing_id) = signal(Option::<String>::None);
    let (resizing_id, set_resizing_id) = signal(Option::<String>::None);

    let (f_name,      set_f_name)      = signal(String::new());
    let (f_address,   set_f_address)   = signal(String::new());
    let (f_port,      set_f_port)      = signal(22i32);
    let (f_username,  set_f_username)  = signal(String::from("root"));
    let (f_auth_method, set_f_auth_method) = signal(String::from("password"));
    let (f_password,  set_f_password)  = signal(Option::<String>::None);
    let (f_ssh_key,   set_f_ssh_key)   = signal(Option::<String>::None);
    let (f_ssh_public_key, set_f_ssh_public_key) = signal(Option::<String>::None);
    let (f_ssh_passphrase, set_f_ssh_passphrase) = signal(Option::<String>::None);
    let (f_notes,     set_f_notes)     = signal(String::new());
    let (f_zfs_pool_size, set_f_zfs_pool_size) = signal(10i32);
    let (f_resize_size, set_f_resize_size) = signal(10i32);
    let (f_storage_device, set_f_storage_device) = signal(Option::<String>::None);

    // Selected host for setup feedback
    let (target_id, set_target_id) = signal(Option::<String>::None);
    
    // Stats polling
    let (stats_map, set_stats_map) = signal(std::collections::HashMap::<String, HostStats>::new());

    // Server actions
    let create_action = ServerAction::<CreateHost>::new();
    let update_action = ServerAction::<UpdateHost>::new();
    let delete_action = ServerAction::<DeleteHost>::new();
    let test_action   = ServerAction::<TestHostConnection>::new();
    let verify_action = ServerAction::<VerifyHostDependencies>::new();
    let setup_action  = ServerAction::<SetupHost>::new();
    let deploy_fox_action = ServerAction::<DeployFox>::new();
    let resize_action = ServerAction::<ResizeHostPool>::new();
    let scan_drives_action = ServerAction::<ListHostDrives>::new();
    let _stats_action = ServerAction::<GetHostStats>::new();

    let user_ctx = expect_context::<crate::app::UserContext>();
    let is_viewer = move || match user_ctx.get() {
        Some(Ok(Some(u))) => u.role == crate::auth::UserRole::Viewer,
        _ => false,
    };

    // Resource — refetches whenever any action version changes
    let hosts_res = Resource::new(
        move || (
            create_action.version().get(),
            update_action.version().get(),
            delete_action.version().get(),
            test_action.version().get(),
            setup_action.version().get(),
            resize_action.version().get(),
            deploy_fox_action.version().get(),
        ),
        |_| async { list_hosts().await },
    );

    // Polling effect
    Effect::new(move |_| {
        let _ = std::time::Duration::from_secs(10);
        let hosts = hosts_res.get();
        if let Some(Ok(list)) = hosts {
            for host in list {
                let id = host.id.clone();
                leptos::task::spawn_local(async move {
                    if let Ok(st) = get_host_stats(id.clone()).await {
                        set_stats_map.update(|m| { m.insert(id, st); });
                    }
                });
            }
        }
    });

    // Reset / close modal
    let do_close = move || {
        set_show_modal.set(false);
        set_editing_host.set(None);
        set_f_name.set(String::new());
        set_f_address.set(String::new());
        set_f_port.set(22);
        set_f_username.set(String::from("root"));
        set_f_auth_method.set(String::from("password"));
        set_f_password.set(None);
        set_f_ssh_key.set(None);
        set_f_ssh_public_key.set(None);
        set_f_ssh_passphrase.set(None);
        set_f_notes.set(String::new());
        set_f_zfs_pool_size.set(100);
        set_f_storage_device.set(None);
    };

    // Form submit
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if let Some(host) = editing_host.get_untracked() {
            update_action.dispatch(UpdateHost {
                id: host.id,
                name: f_name.get_untracked(),
                address: f_address.get_untracked(),
                port: f_port.get_untracked(),
                username: f_username.get_untracked(),
                auth_method: f_auth_method.get_untracked(),
                password: f_password.get_untracked(),
                ssh_key: f_ssh_key.get_untracked(),
                ssh_public_key: f_ssh_public_key.get_untracked(),
                ssh_passphrase: f_ssh_passphrase.get_untracked(),
                notes: f_notes.get_untracked(),
                zfs_pool_size_gb: f_zfs_pool_size.get_untracked(),
                storage_device: f_storage_device.get_untracked(),
            });
        } else {
            create_action.dispatch(CreateHost {
                name: f_name.get_untracked(),
                address: f_address.get_untracked(),
                port: f_port.get_untracked(),
                username: f_username.get_untracked(),
                auth_method: f_auth_method.get_untracked(),
                password: f_password.get_untracked(),
                ssh_key: f_ssh_key.get_untracked(),
                ssh_public_key: f_ssh_public_key.get_untracked(),
                ssh_passphrase: f_ssh_passphrase.get_untracked(),
                notes: f_notes.get_untracked(),
                zfs_pool_size_gb: f_zfs_pool_size.get_untracked(),
                storage_device: f_storage_device.get_untracked(),
            });
        }
        do_close();
    };

    view! {
        <div class="flex flex-col gap-6">

            // ── Page header ──────────────────────────────────────────────────
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-white tracking-tight">"Hypervisor Hosts"</h1>
                    <p class="text-sm text-gray-500 mt-1">"Manage compute hosts and verify connectivity"</p>
                </div>
                <button
                    on:click=move |_| { set_editing_host.set(None); set_show_modal.set(true); }
                    disabled=is_viewer
                    class=move || format!("flex items-center gap-2 px-4 py-2.5 bg-blue-600 hover:bg-blue-500 \
                           text-white text-sm font-semibold rounded-lg transition-colors shadow-lg \
                           shadow-blue-500/20 {}", if is_viewer() { "opacity-50 cursor-not-allowed grayscale" } else { "cursor-pointer" })
                >
                    <span class="text-base leading-none">"+"</span>
                    "Add Host"
                </button>
            </div>

            // ── Error Display ────────────────────────────────────────────────
            {move || {
                let err = setup_action.value().get().and_then(|r| r.err());
                
                err.map(|e| view! {
                    <div class="bg-red-500/10 border border-red-500/20 rounded-lg p-4 flex gap-3 items-start animate-in fade-in slide-in-from-top-2 duration-300">
                        <span class="text-red-400 text-lg">"❌"</span>
                        <div class="flex-1 overflow-hidden">
                            <p class="text-sm font-bold text-red-400 uppercase tracking-wider">"Action Failed"</p>
                            <pre class="text-[10px] text-red-300/80 mt-2 whitespace-pre-wrap font-mono bg-black/20 p-2 rounded border border-red-500/10 max-h-40 overflow-y-auto">
                                {e.to_string()}
                            </pre>
                        </div>
                    </div>
                })
            }}

            // ── Stats row ────────────────────────────────────────────────────
            <Suspense fallback=|| view!{
                <div class="grid grid-cols-4 gap-4">
                    {(0..4).map(|_| view!{<div class="h-24 animate-pulse bg-gray-900 rounded-xl border border-gray-800"></div>}).collect_view()}
                </div>
            }>
                {move || hosts_res.get().map(|r| {
                    let hosts = r.unwrap_or_default();
                    let total   = hosts.len();
                    let online  = hosts.iter().filter(|h| h.status == "online").count();
                    let offline = hosts.iter().filter(|h| h.status == "offline").count();
                    let unknown = hosts.iter().filter(|h| h.status == "unknown").count();
                    view! {
                        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                            <StatCard label="Total Hosts" value=total.to_string()   color="blue"/>
                            <StatCard label="Online"      value=online.to_string()  color="green"/>
                            <StatCard label="Offline"     value=offline.to_string() color="red"/>
                            <StatCard label="Unknown"     value=unknown.to_string() color="gray"/>
                        </div>
                    }.into_any()
                })}
            </Suspense>

            // ── Hosts table ──────────────────────────────────────────────────
            <div class="bg-gray-900/60 border border-gray-800 rounded-xl overflow-hidden shadow-xl">
                <Suspense fallback=|| view!{
                    <div class="flex justify-center items-center py-20 text-gray-500 gap-3">
                        <span class="animate-spin text-xl">"⟳"</span>
                        "Loading hosts…"
                    </div>
                }>
                    {move || hosts_res.get().map(|r: Result<Vec<Host>, ServerFnError>| {
                        let hosts = r.ok().unwrap_or_default();
                        if hosts.is_empty() {
                            view! {
                                <div class="flex flex-col items-center justify-center py-24 gap-4 text-center">
                                    <div class="w-16 h-16 rounded-2xl bg-gray-800 flex items-center justify-center text-3xl">"🖥"</div>
                                    <div>
                                        <p class="text-gray-300 font-semibold">"No hosts yet"</p>
                                        <p class="text-gray-600 text-sm mt-1">"Click \"Add Host\" to register your first hypervisor"</p>
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
                                                <th class="px-6 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">"Address"</th>
                                                <th class="px-6 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">"User"</th>
                                                <th class="px-6 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">"Status"</th>
                                                <th class="px-6 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">"Resources"</th>
                                                <th class="px-6 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">"Dependencies"</th>
                                                <th class="px-6 py-3 text-right text-xs font-semibold text-gray-400 uppercase tracking-wider">"Actions"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-gray-800/60">
                                            {hosts.into_iter().map(|host: Host| {
                                                // Pre-clone everything needed per-row before moving into closures
                                                let h_edit   = host.clone();
                                                let id_del   = host.id.clone();
                                                let id_test  = host.id.clone();
                                                let id_test2 = host.id.clone();
                                                let id_test3 = host.id.clone();
                                                let id_status = host.id.clone();
                                                let name    = host.name.clone();
                                                let addr    = host.address.clone();
                                                let port    = host.port;
                                                let uname   = host.username.clone();
                                                let status  = host.status.clone();
                                                let id_check = host.id.clone();
                                                let id_prov  = host.id.clone();
                                                let id_res_bar = host.id.clone();
                                                let id_resize_btn = host.id.clone();

                                                view! {
                                                    <tr class="hover:bg-gray-800/30 transition-colors">
                                                        <td class="px-6 py-4">
                                                            <span class="font-semibold text-gray-100">{name}</span>
                                                        </td>
                                                        <td class="px-6 py-4">
                                                            <span class="font-mono text-sm text-gray-300">
                                                                {format!("{}:{}", addr, port)}
                                                            </span>
                                                        </td>
                                                        <td class="px-6 py-4">
                                                            <span class="text-sm text-gray-400">{uname}</span>
                                                        </td>
                                                        <td class="px-6 py-4">
                                                            <StatusBadge status=status/>
                                                        </td>
                                                        <td class="px-6 py-4">
                                                            <div class="flex items-center gap-4">
                                                                {move || {
                                                                    let id_res_bar = id_res_bar.clone();
                                                                    let stats = stats_map.get();
                                                                    let s = stats.get(&id_res_bar);
                                                                    view! {
                                                                        <ResourceBar label="CPU" value=s.map(|v| v.cpu_usage).unwrap_or(0.0) color="blue" />
                                                                        <ResourceBar label="RAM" value=s.map(|v| v.mem_usage).unwrap_or(0.0) color="green" />
                                                                        <ResourceBar label="DISK" value=s.map(|v| v.disk_usage).unwrap_or(0.0) color="red" />
                                                                    }
                                                                }}
                                                            </div>
                                                        </td>
                                                        <td class="px-6 py-4">
                                                            {move || {
                                                                let id_check = id_check.clone();
                                                                let id_prov = id_prov.clone();
                                                                let report = verify_action.value().get().and_then(|r| r.ok());
                                                                let is_loading = verify_action.pending().get() && target_id.get().as_deref() == Some(&id_check);
                                                                let is_setting_up = setup_action.pending().get() && target_id.get().as_deref() == Some(&id_prov);
                                                                
                                                                if is_loading {
                                                                    view! { <span class="text-xs text-gray-500 italic animate-pulse">"Checking..."</span> }.into_any()
                                                                } else if is_setting_up {
                                                                    view! { <span class="text-xs text-blue-400 italic animate-pulse">"Setting up..."</span> }.into_any()
                                                                } else if let Some(r) = report {
                                                                    let id_prov_click = id_prov.clone();
                                                                    view! {
                                                                        <div class="flex items-center gap-2">
                                                                            <span class=format!("text-[10px] uppercase font-bold px-1.5 py-0.5 rounded {}", if r.qemu_installed { "bg-green-500/10 text-green-400" } else { "bg-red-500/10 text-red-400" })>"QEMU"</span>
                                                                            <span class=format!("text-[10px] uppercase font-bold px-1.5 py-0.5 rounded {}", if r.docker_installed { "bg-green-500/10 text-green-400" } else { "bg-red-500/10 text-red-400" })>"DOCKER"</span>
                                                                            <span class=format!("text-[10px] uppercase font-bold px-1.5 py-0.5 rounded {}", if r.nvidia_smi_available { "bg-green-500/15 text-green-400" } else { "bg-orange-500/10 text-orange-400" })>"NVIDIA-SMI"</span>
                                                                            <span class=format!("text-[10px] uppercase font-bold px-1.5 py-0.5 rounded {}", if r.nvidia_runtime_configured { "bg-green-500/15 text-green-400" } else { "bg-orange-500/10 text-orange-400" })>"NV-RUNTIME"</span>
                                                                            {(!r.qemu_installed || !r.docker_installed || !r.nvidia_smi_available || !r.nvidia_runtime_configured).then(|| {
                                                                                let id_prov_btn = id_prov_click.clone();
                                                                                view! {
                                                                                    <button 
                                                                                        on:click=move |_| {
                                                                                            set_target_id.set(Some(id_prov_btn.clone()));
                                                                                            setup_action.dispatch(SetupHost { id: id_prov_btn.clone() });
                                                                                        }
                                                                                        disabled=is_viewer
                                                                                        class=move || format!("ml-2 text-[10px] text-blue-400 hover:underline {}", if is_viewer() { "opacity-30 cursor-not-allowed" } else { "cursor-pointer" })
                                                                                    >
                                                                                        "Fix"
                                                                                    </button>
                                                                                }
                                                                            })}
                                                                        </div>
                                                                    }.into_any()
                                                                } else {
                                                                    let id_check_click = id_check.clone();
                                                                    view! {
                                                                        <button 
                                                                            on:click=move |_| {
                                                                                set_target_id.set(Some(id_check_click.clone()));
                                                                                verify_action.dispatch(VerifyHostDependencies { id: id_check_click.clone() });
                                                                            }
                                                                            disabled=is_viewer
                                                                            class=move || format!("text-xs text-gray-500 hover:text-gray-300 flex items-center gap-1.5 {}", if is_viewer() { "opacity-30 cursor-not-allowed pointer-events-none" } else { "cursor-pointer" })
                                                                        >
                                                                            <span class="text-xs">"🔍"</span> "Check"
                                                                        </button>
                                                                    }.into_any()
                                                                }
                                                            }}
                                                        </td>
                                                        <td class="px-6 py-4 text-right">
                                                            <div class="flex items-center justify-end gap-2">
                                                                // Setup
                                                                <button
                                                                    on:click={
                                                                        let id_status = id_status.clone();
                                                                        move |_| {
                                                                            set_target_id.set(Some(id_status.clone()));
                                                                            setup_action.dispatch(SetupHost { id: id_status.clone() });
                                                                        }
                                                                    }
                                                                    disabled=move || is_viewer() || setup_action.pending().get()
                                                                    class=move || format!("px-3 py-1.5 text-xs font-medium rounded-md \
                                                                           bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 \
                                                                           hover:bg-emerald-500/25 transition-colors disabled:opacity-50 {}",
                                                                           if is_viewer() { "opacity-30 grayscale cursor-not-allowed" } else { "cursor-pointer" }
                                                                    )
                                                                >
                                                                    {
                                                                        let id_status = id_status.clone();
                                                                        move || if setup_action.pending().get() && target_id.get().as_deref() == Some(&id_status) {
                                                                            "Setting up…"
                                                                        } else {
                                                                            "Setup"
                                                                        }
                                                                    }
                                                                </button>
                                                                // Test Connection
                                                                <button
                                                                    on:click={
                                                                        let id_test = id_test.clone();
                                                                        move |_| {
                                                                            set_testing_id.set(Some(id_test.clone()));
                                                                            test_action.dispatch(TestHostConnection { id: id_test.clone() });
                                                                        }
                                                                    }
                                                                    class=move || format!("px-3 py-1.5 text-xs font-medium rounded-md \
                                                                           bg-blue-500/10 text-blue-400 border border-blue-500/20 \
                                                                           hover:bg-blue-500/25 transition-colors disabled:opacity-50 {}",
                                                                           if is_viewer() { "opacity-30 grayscale cursor-not-allowed" } else { "cursor-pointer" }
                                                                    )
                                                                    disabled=move || is_viewer() || (test_action.pending().get() && testing_id.get().as_deref() == Some(&id_test2))
                                                                >
                                                                    {
                                                                        let id_test = id_test3.clone();
                                                                        move || if test_action.pending().get() && testing_id.get().as_deref() == Some(&id_test) {
                                                                            "Testing…"
                                                                        } else {
                                                                            "Test"
                                                                        }
                                                                    }
                                                                </button>
                                                                // Deploy Fox
                                                                <button
                                                                    on:click={
                                                                        let id_status = id_status.clone();
                                                                        move |_| {
                                                                            set_target_id.set(Some(id_status.clone()));
                                                                            deploy_fox_action.dispatch(DeployFox { host_id: id_status.clone() });
                                                                        }
                                                                    }
                                                                    class=move || format!("px-3 py-1.5 text-xs font-medium rounded-md \
                                                                           bg-indigo-500/10 text-indigo-400 border border-indigo-500/20 \
                                                                           hover:bg-indigo-500/25 transition-colors disabled:opacity-50 {}",
                                                                           if is_viewer() { "opacity-30 grayscale cursor-not-allowed" } else { "cursor-pointer" }
                                                                    )
                                                                    disabled=move || is_viewer() || deploy_fox_action.pending().get()
                                                                >
                                                                    {
                                                                        let id_status = id_status.clone();
                                                                        move || if deploy_fox_action.pending().get() && target_id.get().as_deref() == Some(&id_status) {
                                                                            "Deploying…"
                                                                        } else {
                                                                            "Deploy Fox"
                                                                        }
                                                                    }
                                                                </button>
                                                                <button
                                                                    on:click=move |_| {
                                                                        set_f_name.set(h_edit.name.clone());
                                                                        set_f_address.set(h_edit.address.clone());
                                                                        set_f_port.set(h_edit.port);
                                                                        set_f_username.set(h_edit.username.clone());
                                                                        set_f_auth_method.set(h_edit.auth_method.clone());
                                                                        set_f_password.set(h_edit.password.clone());
                                                                        set_f_ssh_key.set(h_edit.ssh_key.clone());
                                                                        set_f_ssh_public_key.set(h_edit.ssh_public_key.clone());
                                                                        set_f_ssh_passphrase.set(h_edit.ssh_passphrase.clone());

                                                                        set_f_notes.set(h_edit.notes.clone());
                                                                        set_f_zfs_pool_size.set(h_edit.zfs_pool_size_gb);
                                                                        set_f_storage_device.set(h_edit.storage_device.clone());
                                                                        set_editing_host.set(Some(h_edit.clone()));
                                                                        set_show_modal.set(true);
                                                                    }
                                                                    disabled=is_viewer
                                                                    class=move || format!("px-3 py-1.5 text-xs font-medium rounded-md \
                                                                           bg-gray-700 text-gray-300 hover:bg-gray-600 transition-colors {}",
                                                                           if is_viewer() { "opacity-30 grayscale cursor-not-allowed" } else { "cursor-pointer" }
                                                                    )
                                                                >"Edit"</button>
                                                                <button
                                                                    on:click={
                                                                        let id_resize_btn = id_resize_btn.clone();
                                                                        let current_size = host.zfs_pool_size_gb;
                                                                        move |_| {
                                                                            set_f_resize_size.set(current_size);
                                                                            set_resizing_id.set(Some(id_resize_btn.clone()));
                                                                        }
                                                                    }
                                                                    disabled=is_viewer
                                                                    class=move || format!("px-3 py-1.5 text-xs font-medium rounded-md \
                                                                           bg-amber-500/10 text-amber-400 border border-amber-500/20 \
                                                                           hover:bg-amber-500/25 transition-colors disabled:opacity-50 {}",
                                                                           if is_viewer() { "opacity-30 grayscale cursor-not-allowed" } else { "cursor-pointer" }
                                                                    )
                                                                >"Resize"</button>
                                                                <button
                                                                    on:click=move |_| set_confirm_delete_id.set(Some(id_del.clone()))
                                                                    disabled=is_viewer
                                                                    class=move || format!("px-3 py-1.5 text-xs font-medium rounded-md \
                                                                           bg-red-500/10 text-red-400 border border-red-500/40 \
                                                                           hover:bg-red-500/25 transition-colors {}",
                                                                           if is_viewer() { "opacity-30 grayscale cursor-not-allowed" } else { "cursor-pointer" }
                                                                    )
                                                                >"Delete"</button>
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

            // ── Resize Modal ─────────────────────────────────────────────────
            {move || resizing_id.get().map(|id| view! {
                <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
                    <div class="absolute inset-0 bg-black/70 backdrop-blur-sm" on:click=move |_| set_resizing_id.set(None) />
                    <div class="relative bg-gray-900 border border-gray-700 rounded-2xl shadow-2xl w-full max-w-sm overflow-hidden text-left">
                        <div class="px-6 py-5 border-b border-gray-800 flex justify-between items-center">
                            <h2 class="text-base font-bold text-white uppercase tracking-tight">"Resize Storage Pool"</h2>
                            <button on:click=move |_| set_resizing_id.set(None) class="text-gray-500 hover:text-white transition-colors">"×"</button>
                        </div>
                        <div class="p-6 space-y-4">
                            <p class="text-xs text-gray-400">"Enter the new total size (in GB) for the storage pool. This will expand the ZFS backing file on the host."</p>
                            <div>
                                <label class=label_cls()>"New Size (GB)"</label>
                                <input type="number" min="1"
                                    class=input_cls()
                                    prop:value=move || f_resize_size.get().to_string()
                                    on:input=move |ev| {
                                        if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                            set_f_resize_size.set(v);
                                        }
                                    }
                                />
                            </div>
                            <div class="flex gap-3 pt-2">
                                <button
                                    on:click=move |_| {
                                        resize_action.dispatch(ResizeHostPool { id: id.clone(), new_size_gb: f_resize_size.get_untracked() });
                                        set_resizing_id.set(None);
                                    }
                                    class="flex-1 px-4 py-2 bg-amber-600 hover:bg-amber-500 text-white text-sm font-bold rounded-lg transition-all"
                                >
                                    "Expand Pool"
                                </button>
                                <button
                                    on:click=move |_| set_resizing_id.set(None)
                                    class="flex-1 px-4 py-2 bg-gray-800 hover:bg-gray-700 text-gray-300 text-sm font-bold rounded-lg transition-all"
                                >
                                    "Cancel"
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            })}

            // ── Add / Edit Modal ─────────────────────────────────────────────
            {move || show_modal.get().then(|| view! {
                <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
                    <div
                        class="absolute inset-0 bg-black/70 backdrop-blur-sm"
                        on:click=move |_| do_close()
                    />
                    <div class="relative bg-gray-900 border border-gray-700 rounded-2xl shadow-2xl w-full max-w-lg overflow-hidden flex flex-col max-h-[90vh] text-left">
                        <div class="px-6 py-5 border-b border-gray-800 flex justify-between items-center bg-gray-900 shrink-0">
                            <h2 class="text-lg font-bold text-white uppercase tracking-tight">
                                {move || if editing_host.get().is_some() { "Edit Host" } else { "Add New Host" }}
                            </h2>
                            <button on:click=move |_| do_close() class="text-gray-500 hover:text-white transition-colors text-2xl leading-none">"×"</button>
                        </div>

                        <form on:submit=on_submit class="flex-1 flex flex-col overflow-hidden">
                            <div class="flex-1 overflow-y-auto p-6 space-y-8 custom-scrollbar">
                                // Name
                                <div>
                                    <label class=label_cls()>"Name"</label>
                                    <input type="text" placeholder="e.g. Host-01" 
                                        class=input_cls()
                                        prop:value=f_name
                                        on:input=move |ev| set_f_name.set(event_target_value(&ev))
                                    />
                                </div>

                                // Address + Port
                                <div class="grid grid-cols-2 gap-4">
                                    <div>
                                        <label class=label_cls()>"Address"</label>
                                        <input type="text" placeholder="192.168.1.10"
                                            class=input_cls()
                                            prop:value=f_address
                                            on:input=move |ev| set_f_address.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label class=label_cls()>"SSH Port"</label>
                                        <input type="number" 
                                            class=input_cls()
                                            prop:value=move || f_port.get().to_string()
                                            on:input=move |ev| {
                                                if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                                    set_f_port.set(v);
                                                }
                                            }
                                        />
                                    </div>
                                </div>

                                // Auth Method
                                <div class="bg-gray-800/50 p-4 rounded-xl space-y-4">
                                    <div>
                                        <label class=label_cls()>"Authentication Method"</label>
                                        <select 
                                            class=input_cls()
                                            on:change=move |ev| set_f_auth_method.set(event_target_value(&ev))
                                        >
                                            <option value="password" selected=move || f_auth_method.get() == "password">"Password"</option>
                                            <option value="key" selected=move || f_auth_method.get() == "key">"SSH Key"</option>
                                        </select>
                                    </div>

                                    <div>
                                        <label class=label_cls()>"Username"</label>
                                        <input type="text" 
                                            class=input_cls()
                                            prop:value=f_username
                                            on:input=move |ev| set_f_username.set(event_target_value(&ev))
                                        />
                                    </div>

                                    {move || if f_auth_method.get() == "password" {
                                        view! {
                                            <div>
                                                <label class=label_cls()>"Password"</label>
                                                <input type="password" 
                                                    class=input_cls()
                                                    prop:value=move || f_password.get().clone().unwrap_or_default()
                                                    on:input=move |ev| set_f_password.set(Some(event_target_value(&ev)))
                                                />
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="space-y-4">
                                                <div>
                                                    <label class=label_cls()>"SSH Private Key"</label>
                                                    <textarea 
                                                        class=format!("{} h-32 font-mono text-[10px]", input_cls())
                                                        prop:value=move || f_ssh_key.get().clone().unwrap_or_default()
                                                        on:input=move |ev| set_f_ssh_key.set(Some(event_target_value(&ev)))
                                                    />
                                                </div>
                                                <div>
                                                    <label class=label_cls()>"Key Passphrase (Optional)"</label>
                                                    <input type="password" 
                                                        class=input_cls()
                                                        prop:value=move || f_ssh_passphrase.get().clone().unwrap_or_default()
                                                        on:input=move |ev| set_f_ssh_passphrase.set(Some(event_target_value(&ev)))
                                                    />
                                                </div>
                                            </div>
                                        }.into_any()
                                    }}
                                </div>

                                // Notes
                                <div>
                                    <label class=label_cls()>"Notes"</label>
                                    <textarea 
                                        class=format!("{} h-20", input_cls())
                                        prop:value=f_notes
                                        on:input=move |ev| set_f_notes.set(event_target_value(&ev))
                                    />
                                </div>

                                // Storage Configuration
                                <div class="bg-blue-500/5 border border-blue-500/20 p-5 rounded-2xl relative">
                                    <div class="absolute -top-3 left-4 bg-gray-900 px-2 py-0.5 border border-blue-500/20 rounded-md">
                                        <span class="text-[10px] font-bold text-blue-400 uppercase tracking-widest">"Storage Configuration"</span>
                                    </div>
                                    
                                    <div class="space-y-4">
                                        <div class="flex items-center justify-between">
                                            <p class="text-[10px] text-gray-500 uppercase font-bold tracking-wider">"Storage Device"</p>
                                            {move || if editing_host.get().is_some() {
                                                let host = editing_host.get().unwrap();
                                                view! {
                                                    <button 
                                                        type="button"
                                                        on:click=move |_| { scan_drives_action.dispatch(ListHostDrives { host_id: host.id.clone() }); }
                                                        disabled=scan_drives_action.pending()
                                                        class="text-[10px] text-blue-400 hover:text-blue-300 font-bold uppercase tracking-wider disabled:opacity-50"
                                                    >
                                                        {move || if scan_drives_action.pending().get() { "Scanning..." } else { "Scan for Drives" }}
                                                    </button>
                                                }.into_any()
                                            } else {
                                                view! { <span class="text-[10px] text-gray-500 italic">"Save host to scan for drives"</span> }.into_any()
                                            }}
                                        </div>
                                        
                                        {move || {
                                            let res = scan_drives_action.value().get();
                                            let drives = match res {
                                                Some(Ok(cv)) => cv.clone(),
                                                _ => vec![]
                                            };
                                            let drives_for_check = drives.clone();
                                            view! {
                                                <div class="space-y-3">
                                                    <select 
                                                        class=input_cls()
                                                        on:change=move |ev| {
                                                            let val = event_target_value(&ev);
                                                            if val == "__FILE__" {
                                                                set_f_storage_device.set(None);
                                                            } else {
                                                                set_f_storage_device.set(Some(val));
                                                            }
                                                        }
                                                    >
                                                        <option value="__FILE__" selected=move || f_storage_device.get().is_none()>"Sparse File (default 100GB)"</option>
                                                        
                                                        // Always include current storage device if it's not a file
                                                        {move || {
                                                            let current = f_storage_device.get();
                                                            let drives_list = drives_for_check.clone();
                                                            if let Some(c) = current {
                                                                // If current device isn't in the scanned list, show it anyway
                                                                if !drives_list.iter().any(|d| d.name == c) {
                                                                    return view! {
                                                                        <option value=c.clone() selected=true>{format!("{} (Current)", c)}</option>
                                                                    }.into_any();
                                                                }
                                                            }
                                                            view! {}.into_any()
                                                        }}

                                                        {drives.into_iter().map(|d| {
                                                            let name = d.name.clone();
                                                            let tran_prefix = d.tran.as_ref().map(|t| format!("[{}] ", t.to_uppercase())).unwrap_or_default();
                                                            let vendor_prefix = d.vendor.as_ref().map(|v| format!("{} ", v)).unwrap_or_else(|| "".into());
                                                            let label = format!("{}{}{} ({} {})", 
                                                                tran_prefix, 
                                                                vendor_prefix,
                                                                d.model.clone().unwrap_or_else(|| "Unknown".into()), 
                                                                d.size, 
                                                                d.name
                                                            );
                                                            let is_selected = f_storage_device.get().as_deref() == Some(&name);
                                                            view! {
                                                                <option value=name.clone() selected=is_selected>{label}</option>
                                                            }
                                                        }).collect_view()}
                                                    </select>
                                                    
                                                    {move || if f_storage_device.get().is_some() {
                                                        view! {
                                                            <div class="bg-amber-500/10 border border-amber-500/20 rounded-lg p-3">
                                                                <p class="text-[10px] text-amber-400 font-bold uppercase tracking-tight mb-1">"⚠️ Raw Disk Warning"</p>
                                                                <p class="text-[10px] text-amber-200/70 leading-relaxed">
                                                                    "Using a raw disk will format the entire drive. "
                                                                    <span class="font-bold underline">"All existing data on the selected drive will be destroyed."</span>
                                                                </p>
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <div>
                                                                <label class=label_cls()>"Maximum Pool Size (GB)"</label>
                                                                <input type="number" min="1"
                                                                    class=input_cls()
                                                                    prop:value=move || f_zfs_pool_size.get().to_string()
                                                                    on:input=move |ev| {
                                                                        if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                                                            set_f_zfs_pool_size.set(v);
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                        }.into_any()
                                                    }}
                                                </div>
                                            }
                                        }}
                                    </div>
                                </div>
                            </div>

                            <div class="px-6 py-4 bg-gray-900 border-t border-gray-800 flex justify-end gap-3 shrink-0">
                                <button type="button" on:click=move |_| do_close()
                                    class="px-4 py-2 text-sm text-gray-400 hover:text-white transition-colors">
                                    "Cancel"
                                </button>
                                <button type="submit"
                                    class="px-6 py-2 bg-blue-600 hover:bg-blue-500 text-white \
                                           text-sm font-bold rounded-xl transition-all shadow-lg shadow-blue-500/20">
                                    {move || if editing_host.get().is_some() { "Update Host" } else { "Create Host" }}
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            })}

            // ── Delete Confirm Modal ─────────────────────────────────────────
            {move || confirm_delete_id.get().map(|del_id| {
                let del_id2 = del_id.clone();
                view! {
                    <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
                        <div class="absolute inset-0 bg-black/70 backdrop-blur-sm" on:click=move |_| set_confirm_delete_id.set(None) />
                        <div class="relative bg-gray-900 border border-red-900/40 rounded-2xl shadow-2xl p-6 max-w-sm w-full text-left">
                            <div class="flex items-center gap-3 mb-3">
                                <span class="text-2xl">"⚠️"</span>
                                <h2 class="text-base font-bold text-white uppercase">"Delete Host"</h2>
                            </div>
                            <p class="text-gray-400 text-sm mb-6">
                                "This will permanently remove the host and all associated records."
                            </p>
                            <div class="flex gap-3 justify-end">
                                <button on:click=move |_| set_confirm_delete_id.set(None)
                                    class="px-4 py-2 text-sm text-gray-400 hover:text-white transition-colors">
                                    "Cancel"
                                </button>
                                <button on:click=move |_| {
                                    delete_action.dispatch(DeleteHost { id: del_id2.clone() });
                                    set_confirm_delete_id.set(None);
                                }
                                    class="px-4 py-2 bg-red-600 hover:bg-red-500 text-white \
                                           text-sm font-bold rounded-lg transition-colors shadow-lg shadow-red-500/20">
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
