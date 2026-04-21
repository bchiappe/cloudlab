use leptos::prelude::*;
use crate::vms::*;
use crate::hosts::list_hosts;

// ─── Status Badge ─────────────────────────────────────────────────────────────

#[component]
fn StatusBadge(status: String) -> impl IntoView {
    let (bg, dot, label) = match status.as_str() {
        "running" => ("bg-green-500/15 border-green-500/30 text-green-400", "bg-green-400", "Running"),
        "stopped" => ("bg-red-500/15 border-red-500/30 text-red-400", "bg-red-400", "Stopped"),
        "paused"  => ("bg-yellow-500/15 border-yellow-500/30 text-yellow-400", "bg-yellow-400", "Paused"),
        _         => ("bg-gray-500/15 border-gray-500/30 text-gray-400", "bg-gray-400", "Unknown"),
    };
    view! {
        <span class=format!("inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium border {}", bg)>
            <span class=format!("w-1.5 h-1.5 rounded-full {}", dot)></span>
            {label}
        </span>
    }
}

// ─── OS Badge ───────────────────────────────────────────────────────────────

#[component]
fn OSBadge(os_type: String) -> impl IntoView {
    let (cls, label) = match os_type.as_str() {
        "linux"   => ("bg-orange-500/15 border-orange-500/30 text-orange-400", "Linux"),
        "windows" => ("bg-blue-500/15 border-blue-500/30 text-blue-400", "Windows"),
        "macos"   => ("bg-gray-500/15 border-gray-500/30 text-gray-100", "macOS"),
        _         => ("bg-cyan-500/15 border-cyan-500/30 text-cyan-400", "Other"),
    };
    view! {
        <span class=format!("inline-flex px-2.5 py-1 rounded-full text-xs font-medium border {}", cls)>
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

// ─── VMs Page ────────────────────────────────────────────────────────────────

#[component]
pub fn VMsPage() -> impl IntoView {
    // Modal / form state
    let (show_modal, set_show_modal) = signal(false);
    let (editing_vm, set_editing_vm) = signal(Option::<VM>::None);
    let (confirm_delete_id, set_confirm_delete_id) = signal(Option::<String>::None);
    
    let (f_name,      set_f_name)      = signal(String::new());
    let (f_host_id,   set_f_host_id)   = signal(String::new());
    let (f_cpu,       set_f_cpu)       = signal(1i32);
    let (f_memory,    set_f_memory)    = signal(1024i32);
    let (f_os_type,   set_f_os_type)   = signal(String::from("linux"));

    // Server actions
    let create_action = ServerAction::<CreateVM>::new();
    let update_action = ServerAction::<UpdateVM>::new();
    let delete_action = ServerAction::<DeleteVM>::new();
    let power_action  = ServerAction::<ToggleVMPower>::new();
    let mount_action = ServerAction::<MountISO>::new();
    let unmount_action = ServerAction::<UnmountISO>::new();

    // Resource — refetches whenever any action version changes
    let vms_res = Resource::new(
        move || (
            create_action.version().get(),
            update_action.version().get(),
            delete_action.version().get(),
            power_action.version().get(),
        ),
        |_| async { list_vms().await },
    );

    // Also fetch hosts for the select dropdown
    let hosts_res = Resource::new(|| (), |_| async { list_hosts().await });

    // ISO state
    let (show_iso_modal, set_show_iso_modal) = signal(Option::<VM>::None);
    let isos_res = Resource::new(
        move || show_iso_modal.get().map(|v| v.host_id),
        |host_id| async move {
            if let Some(hid) = host_id {
                list_isos(hid).await
            } else {
                Ok(vec![])
            }
        }
    );

    let user_ctx = expect_context::<crate::app::UserContext>();
    let is_viewer = move || match user_ctx.get() {
        Some(Ok(Some(u))) => u.role == crate::auth::UserRole::Viewer,
        _ => false,
    };

    // Reset / close modal
    let do_close = move || {
        set_show_modal.set(false);
        set_editing_vm.set(None);
        set_f_name.set(String::new());
        set_f_host_id.set(String::new());
        set_f_cpu.set(1);
        set_f_memory.set(1024);
        set_f_os_type.set(String::from("linux"));
    };

    // Form submit
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if let Some(vm) = editing_vm.get_untracked() {
            update_action.dispatch(UpdateVM {
                id: vm.id,
                host_id: f_host_id.get_untracked(),
                name: f_name.get_untracked(),
                cpu: f_cpu.get_untracked(),
                memory_mb: f_memory.get_untracked(),
                os_type: f_os_type.get_untracked(),
            });
        } else {
            create_action.dispatch(CreateVM {
                host_id: f_host_id.get_untracked(),
                name: f_name.get_untracked(),
                cpu: f_cpu.get_untracked(),
                memory_mb: f_memory.get_untracked(),
                os_type: f_os_type.get_untracked(),
            });
        }
        do_close();
    };

    view! {
        <div class="flex flex-col gap-6">

            // ── Page header ──────────────────────────────────────────────────
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-white tracking-tight">"Virtual Machines"</h1>
                    <p class="text-sm text-gray-500 mt-1">"Manage and monitor virtualized workloads"</p>
                </div>
                <button
                    on:click=move |_| { set_editing_vm.set(None); set_show_modal.set(true); }
                    disabled=is_viewer
                    class=move || format!("flex items-center gap-2 px-4 py-2.5 bg-blue-600 hover:bg-blue-500 \
                           text-white text-sm font-semibold rounded-lg transition-colors shadow-lg \
                           shadow-blue-500/20 {}", if is_viewer() { "opacity-50 cursor-not-allowed grayscale" } else { "cursor-pointer" })
                >
                    <span class="text-base leading-none">"+"</span>
                    "Create VM"
                </button>
            </div>

            // ── Stats row ────────────────────────────────────────────────────
            <Suspense fallback=|| view!{
                <div class="grid grid-cols-4 gap-4">
                    {(0..4).map(|_| view!{<div class="h-24 animate-pulse bg-gray-900 rounded-xl border border-gray-800"></div>}).collect_view()}
                </div>
            }>
                {move || vms_res.get().map(|r| {
                    let vms = r.unwrap_or_default();
                    let total   = vms.len();
                    let running = vms.iter().filter(|v| v.status == "running").count();
                    let stopped = vms.iter().filter(|v| v.status == "stopped").count();
                    let total_ram = vms.iter().map(|v| v.memory_mb as u64).sum::<u64>();
                    view! {
                        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                            <StatCard label="Total VMs" value=total.to_string()   color="blue"/>
                            <StatCard label="Running"   value=running.to_string() color="green"/>
                            <StatCard label="Stopped"   value=stopped.to_string() color="red"/>
                            <StatCard label="Total Allocated RAM" value=format!("{:.1} GB", total_ram as f64 / 1024.0) color="gray"/>
                        </div>
                    }.into_any()
                })}
            </Suspense>

            // ── VMs table ────────────────────────────────────────────────────
            <div class="bg-gray-900/60 border border-gray-800 rounded-xl overflow-hidden shadow-xl">
                <Suspense fallback=|| view!{
                    <div class="flex justify-center items-center py-20 text-gray-500 gap-3">
                        <span class="animate-spin text-xl">"⟳"</span>
                        "Loading virtual machines…"
                    </div>
                }>
                    {move || vms_res.get().map(|r| {
                        let vms = r.unwrap_or_default();
                        if vms.is_empty() {
                            view! {
                                <div class="flex flex-col items-center justify-center py-24 gap-4 text-center">
                                    <div class="w-16 h-16 rounded-2xl bg-gray-800 flex items-center justify-center text-3xl">"📦"</div>
                                    <div>
                                        <p class="text-gray-300 font-semibold">"No virtual machines yet"</p>
                                        <p class="text-gray-600 text-sm mt-1">"Click \"Create VM\" to deploy your first workload"</p>
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
                                                <th class="px-6 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">"Host"</th>
                                                <th class="px-6 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">"Specs"</th>
                                                <th class="px-6 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">"OS"</th>
                                                <th class="px-6 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider">"Status"</th>
                                                <th class="px-6 py-3 text-right text-xs font-semibold text-gray-400 uppercase tracking-wider">"Actions"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-gray-800/60">
                                            {vms.into_iter().map(|vm| {
                                                let vm_iso = vm.clone();
                                                let vm_edit = vm.clone();
                                                let id_pwr = vm.id.clone();
                                                let id_del = vm.id.clone();
                                                let is_running = vm.status == "running";
                                                
                                                view! {
                                                    <tr class="hover:bg-gray-800/30 transition-colors">
                                                        <td class="px-6 py-4">
                                                            <div class="flex flex-col">
                                                                <span class="font-semibold text-gray-100">{vm.name.clone()}</span>
                                                                <span class="text-[10px] text-gray-600 font-mono tracking-tighter truncate max-w-[100px]">{vm.id.clone()}</span>
                                                            </div>
                                                        </td>
                                                        <td class="px-6 py-4">
                                                            <div class="flex items-center gap-1.5 text-sm text-gray-400">
                                                                <span class="text-xs">"🔗"</span>
                                                                {vm.host_name.clone()}
                                                            </div>
                                                        </td>
                                                        <td class="px-6 py-4 text-sm text-gray-300">
                                                            <div class="flex flex-col gap-0.5">
                                                                 <span>{format!("{} vCPU", vm.cpu)}</span>
                                                                 <span class="text-xs text-gray-500">{format!("{:.1} GB RAM", vm.memory_mb as f64 / 1024.0)}</span>
                                                                 {vm.iso_volume_id.as_ref().map(|iso| view! {
                                                                     <span class="text-[10px] text-blue-400 font-mono mt-1 flex items-center gap-1">"💿 " {iso.clone()}</span>
                                                                 })}
                                                            </div>
                                                        </td>
                                                        <td class="px-6 py-4"><OSBadge os_type=vm.os_type.clone()/></td>
                                                        <td class="px-6 py-4">
                                                            <StatusBadge status=vm.status.clone()/>
                                                        </td>
                                                        <td class="px-6 py-4">
                                                         <div class="flex items-center justify-end gap-2">
                                                                // Power Toggle
                                                                <button
                                                                    on:click={
                                                                        let id = id_pwr.clone();
                                                                        move |_| { power_action.dispatch(ToggleVMPower { id: id.clone() }); }
                                                                    }
                                                                    disabled=is_viewer
                                                                    class=move || format!("p-2 rounded-md transition-colors {} {}", 
                                                                        if is_running { "text-red-400 hover:bg-red-400/10" } else { "text-green-400 hover:bg-green-400/10" },
                                                                        if is_viewer() { "opacity-30 grayscale cursor-not-allowed" } else { "cursor-pointer" }
                                                                    )
                                                                    title=if is_running { "Stop" } else { "Start" }
                                                                >
                                                                    {if is_running { "■" } else { "▶" }}
                                                                </button>
                                                                
                                                                 // Console
                                                                 <button
                                                                     on:click=move |_| {
                                                                         let id = id_pwr.clone();
                                                                         leptos::task::spawn_local(async move {
                                                                            if let Ok(url) = get_vm_console(id).await {
                                                                                let _ = window().open_with_url_and_target(&url, "_blank");
                                                                            }
                                                                         });
                                                                     }
                                                                     class="p-2 text-blue-400 hover:text-blue-300 hover:bg-blue-400/10 rounded-md transition-colors"
                                                                     title="Open Console"
                                                                 >
                                                                     "📺"
                                                                 </button>

                                                                 // ISO
                                                                 <button
                                                                     on:click=move |_| { set_show_iso_modal.set(Some(vm_iso.clone())); }
                                                                     class="p-2 text-cyan-400 hover:text-cyan-300 hover:bg-cyan-400/10 rounded-md transition-colors"
                                                                     title="Manage ISO"
                                                                 >
                                                                     "💿"
                                                                 </button>
                                                                 
                                                                 // Edit
                                                                <button
                                                                    on:click=move |_| {
                                                                        set_f_name.set(vm_edit.name.clone());
                                                                        set_f_host_id.set(vm_edit.host_id.clone());
                                                                        set_f_cpu.set(vm_edit.cpu);
                                                                        set_f_memory.set(vm_edit.memory_mb);
                                                                        set_f_os_type.set(vm_edit.os_type.clone());
                                                                        set_editing_vm.set(Some(vm_edit.clone()));
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
                    <div class="relative bg-gray-900 border border-gray-700 rounded-2xl shadow-2xl w-full max-w-lg overflow-hidden flex flex-col">
                        <div class="flex items-center justify-between px-6 py-5 border-b border-gray-800">
                            <h2 class="text-base font-bold text-white">
                                {move || if editing_vm.get().is_some() { "Edit Virtual Machine" } else { "Create Virtual Machine" }}
                            </h2>
                            <button on:click=move |_| do_close()
                                class="text-gray-500 hover:text-white text-2xl leading-none transition-colors">"×"</button>
                        </div>

                        <form on:submit=on_submit class="p-6 space-y-4">
                            // Name
                            <div>
                                <label class=label_cls()>"VM Name"</label>
                                <input type="text" required placeholder="e.g. web-server-01"
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
                                    <option value="" disabled selected=move || f_host_id.get().is_empty()>"Select a host..."</option>
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

                            // Resource Configuration
                            <div class="grid grid-cols-2 gap-4">
                                <div>
                                    <label class=label_cls()>"vCPUs"</label>
                                    <input type="number" min="1" max="128" required
                                        class=input_cls()
                                        prop:value=move || f_cpu.get().to_string()
                                        on:input=move |ev| {
                                            if let Ok(val) = event_target_value(&ev).parse::<i32>() {
                                                set_f_cpu.set(val);
                                            }
                                        }
                                    />
                                </div>
                                <div>
                                    <label class=label_cls()>"RAM (MB)"</label>
                                    <input type="number" min="128" step="128" required
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

                            // OS Type
                            <div>
                                <label class=label_cls()>"OS Family"</label>
                                <div class="grid grid-cols-3 gap-2">
                                    {["linux", "windows", "macos"].into_iter().map(|os| {
                                        let os_clone = os.to_string();
                                        let is_sel = move || f_os_type.get() == os;
                                        view! {
                                            <button
                                                type="button"
                                                on:click=move |_| set_f_os_type.set(os_clone.clone())
                                                class=move || format!(
                                                    "px-3 py-2 text-xs font-semibold rounded-lg border transition-all {}",
                                                    if is_sel() { 
                                                        "bg-blue-600/20 border-blue-500 text-blue-400 shadow-[0_0_12px_rgba(59,130,246,0.3)]" 
                                                    } else { 
                                                        "bg-gray-800 border-gray-700 text-gray-400 hover:border-gray-500" 
                                                    }
                                                )
                                            >
                                                {os.to_uppercase()}
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
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
                                    {move || if editing_vm.get().is_some() { "Save Changes" } else { "Create VM" }}
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            })}

             // ── ISO Modal ───────────────────────────────────────────────────
             {move || show_iso_modal.get().map(|vm| {
                let id_base = vm.id.clone();
                let current_iso = vm.iso_volume_id.clone();

                let id_unmount = id_base.clone();
                let id_mount_base = id_base.clone();

                view! {
                    <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
                        <div class="absolute inset-0 bg-black/70 backdrop-blur-sm" on:click=move |_| set_show_iso_modal.set(None)/>
                        <div class="relative bg-gray-900 border border-gray-700 rounded-2xl shadow-2xl w-full max-w-md overflow-hidden">
                            <div class="flex items-center justify-between px-6 py-5 border-b border-gray-800">
                                <h2 class="text-base font-bold text-white">"Manage ISO Media"</h2>
                                <button on:click=move |_| set_show_iso_modal.set(None)
                                    class="text-gray-500 hover:text-white text-2xl leading-none">"×"</button>
                            </div>
                            
                            <div class="p-6 space-y-4">
                                <div class="bg-blue-500/5 border border-blue-500/20 rounded-lg p-3">
                                    <p class="text-xs text-blue-300 font-semibold uppercase tracking-wider mb-1">"Target VM"</p>
                                    <p class="text-sm text-white font-bold">{vm.name.clone()}</p>
                                </div>

                                {match current_iso {
                                    Some(iso) => view! {
                                        <div class="flex items-center justify-between p-3 bg-gray-800 rounded-lg border border-gray-700">
                                            <div class="flex items-center gap-3">
                                                <span class="text-xl">"💿"</span>
                                                <div>
                                                    <p class="text-xs text-gray-500">"Currently Mounted"</p>
                                                    <p class="text-sm text-white font-mono">{iso}</p>
                                                </div>
                                            </div>
                                            <button 
                                                on:click=move |_| {
                                                    let id = id_unmount.clone();
                                                    unmount_action.dispatch(UnmountISO { id });
                                                    set_show_iso_modal.set(None);
                                                }
                                                class="px-3 py-1 bg-red-600/20 text-red-400 border border-red-500/30 rounded text-xs font-bold hover:bg-red-600/30 transition-all"
                                            >
                                                "EJECT"
                                            </button>
                                        </div>
                                    }.into_any(),
                                    None => view! {
                                        <div class="p-3 bg-gray-800/40 border border-dashed border-gray-700 rounded-lg text-center">
                                            <p class="text-sm text-gray-500">"No ISO currently mounted"</p>
                                        </div>
                                    }.into_any()
                                }}

                                <div class="flex items-center justify-between gap-2 mb-2">
                                    <label class=label_cls()>"Available Images"</label>
                                    <a 
                                        href="/storage" 
                                        class="text-[10px] text-blue-400 hover:text-blue-300 font-bold uppercase tracking-widest flex items-center gap-1 group transition-all"
                                    >
                                        "Browse Repository"
                                        <span class="group-hover:translate-x-0.5 transition-transform">"→"</span>
                                    </a>
                                </div>
                                <Suspense fallback=|| view! { <div class="h-20 animate-pulse bg-gray-800 rounded-lg"/> }>
                                        {
                                            let id_mount_base = id_mount_base.clone();
                                            move || isos_res.get().map(|r| {
                                                let isos = r.unwrap_or_default();
                                                if isos.is_empty() {
                                                    view! { <p class="text-xs text-gray-600 italic">"No ISOs found in /mnt/isos/"</p> }.into_any()
                                                } else {
                                                    let id_mount = id_mount_base.clone();
                                                    view! {
                                                        <div class="grid grid-cols-1 gap-2 max-h-48 overflow-y-auto pr-2 custom-scrollbar">
                                                            {isos.into_iter().map(move |iso| {
                                                                let id = id_mount.clone();
                                                                let iso_name = iso.clone();
                                                                view! {
                                                                    <button 
                                                                        on:click=move |_| {
                                                                            let id = id.clone();
                                                                            let iso_name = iso_name.clone();
                                                                            mount_action.dispatch(MountISO { id, iso_name });
                                                                            set_show_iso_modal.set(None);
                                                                        }
                                                                        class="flex items-center justify-between p-3 bg-gray-800 hover:bg-gray-700 border border-gray-700 rounded-lg text-left transition-all group"
                                                                    >
                                                                        <span class="text-sm text-gray-300 group-hover:text-white truncate">{iso}</span>
                                                                        <span class="text-xs text-blue-500 font-bold opacity-0 group-hover:opacity-100 transition-opacity">"MOUNT →"</span>
                                                                    </button>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    }.into_any()
                                                }
                                            })
                                        }
                                    </Suspense>
                            </div>
                            
                            <div class="px-6 py-5 bg-gray-800/30 border-t border-gray-800 text-center">
                                <p class="text-[10px] text-gray-500">"All media is stored in the cluster-wide Linstor volume mounted at /mnt/isos"</p>
                            </div>
                        </div>
                    </div>
                }
            })}

            // ── Delete Confirm Modal ──
            {move || confirm_delete_id.get().map(|del_id| {
                let id_clone = del_id.clone();
                view! {
                    <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
                        <div class="absolute inset-0 bg-black/70 backdrop-blur-sm"/>
                        <div class="relative bg-gray-900 border border-red-900/40 rounded-2xl shadow-2xl p-6 max-w-sm w-full">
                            <div class="flex items-center gap-3 mb-3">
                                <span class="bg-red-500/20 p-2 rounded-lg text-red-500 text-xl">"⚠"</span>
                                <h2 class="text-base font-bold text-white">"Delete Virtual Machine"</h2>
                            </div>
                            <p class="text-gray-400 text-sm mb-6 leading-relaxed">
                                "Are you sure you want to delete this VM? All virtual disk data and configuration will be permanently destroyed."
                            </p>
                            <div class="flex gap-3 justify-end">
                                <button on:click=move |_| set_confirm_delete_id.set(None)
                                    class="px-4 py-2 text-sm text-gray-400 hover:text-white transition-colors">
                                    "Cancel"
                                </button>
                                <button on:click=move |_| {
                                    delete_action.dispatch(DeleteVM { id: id_clone.clone() });
                                    set_confirm_delete_id.set(None);
                                }
                                    class="px-4 py-2 bg-red-600 hover:bg-red-500 text-white \
                                           text-sm font-semibold rounded-lg transition-colors">
                                    "Delete VM"
                                </button>
                            </div>
                        </div>
                    </div>
                }
            })}
        </div>
    }
}
