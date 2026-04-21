use leptos::prelude::*;
use crate::backups::*;
use crate::vms::list_vms;
use crate::containers::list_containers;
use crate::databases::list_databases;
use crate::components::icons::{CloudArrowUp, Plus, Trash, Play};

#[component]
pub fn BackupsPage() -> impl IntoView {
    let (show_modal, set_show_modal) = signal(false);
    
    let backups_res = Resource::new(|| (), |_| async { list_backups().await });
    
    // Resources for the wizard
    let vms_res = Resource::new(|| (), |_| async { list_vms().await });
    let containers_res = Resource::new(|| (), |_| async { list_containers().await });
    let databases_res = Resource::new(|| (), |_| async { list_databases().await });

    let create_action = ServerAction::<CreateBackupConfig>::new();
    let run_action = ServerAction::<RunBackupNow>::new();
    let delete_action = ServerAction::<DeleteBackupConfig>::new();

    let user_ctx = expect_context::<crate::app::UserContext>();
    let is_viewer = move || match user_ctx.get() {
        Some(Ok(Some(u))) => u.role == crate::auth::UserRole::Viewer,
        _ => false,
    };

    view! {
        <div class="space-y-6 animate-in fade-in duration-500">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold tracking-tight text-gray-100">"Backup & Recovery"</h1>
                    <p class="text-gray-400 mt-1">"Protect your infrastructure with automated snapshots and offsite replication."</p>
                </div>
                <button 
                    on:click=move |_| set_show_modal.set(true)
                    disabled=is_viewer
                    class=move || format!("flex items-center gap-2 px-4 py-2 bg-purple-600 hover:bg-purple-500 text-white rounded-lg font-semibold transition-all shadow-lg shadow-purple-900/20 {}",
                        if is_viewer() { "opacity-50 cursor-not-allowed grayscale" } else { "cursor-pointer" }
                    )
                >
                    <Plus class="w-4 h-4"/>
                    "Create Backup Task"
                </button>
            </div>

            <div class="grid grid-cols-1 gap-6">
                <div class="bg-gray-900/50 backdrop-blur-sm border border-gray-800 rounded-xl overflow-hidden">
                    <table class="w-full text-left border-collapse">
                        <thead>
                            <tr class="bg-gray-800/50 text-gray-400 text-xs uppercase tracking-wider">
                                <th class="px-6 py-4 font-semibold">"Task Name"</th>
                                <th class="px-6 py-4 font-semibold">"Source"</th>
                                <th class="px-6 py-4 font-semibold">"Schedule"</th>
                                <th class="px-6 py-4 font-semibold">"Last Run"</th>
                                <th class="px-6 py-4 font-semibold">"Status"</th>
                                <th class="px-6 py-4 font-semibold text-right">"Actions"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-800">
                            <Suspense fallback=|| view! { <tr><td colspan="6" class="px-6 py-8 text-center text-gray-500">"Loading backup tasks..."</td></tr> }>
                                {move || backups_res.get().map(|res| match res {
                                    Ok(backups) if backups.is_empty() => view! {
                                        <tr><td colspan="6" class="px-6 py-12 text-center text-gray-500 italic">"No backup tasks configured."</td></tr>
                                    }.into_any(),
                                    Ok(backups) => backups.into_iter().map(|bk| {
                                        let id_run = bk.id.clone();
                                        let id_del = bk.id.clone();
                                        view! {
                                            <tr class="hover:bg-gray-800/30 transition-colors group">
                                                <td class="px-6 py-4">
                                                    <div class="flex items-center gap-3">
                                                        <CloudArrowUp class="w-5 h-5 text-purple-400"/>
                                                        <span class="font-semibold text-gray-100">{bk.name}</span>
                                                    </div>
                                                </td>
                                                <td class="px-6 py-4 text-sm">
                                                    <span class="text-gray-400 capitalize">{bk.source_type}</span>
                                                    <span class="mx-2 text-gray-700">"|"</span>
                                                    <span class="text-gray-300 font-mono text-xs">{bk.source_id.chars().take(8).collect::<String>()}</span>
                                                </td>
                                                <td class="px-6 py-4 text-xs font-semibold text-gray-400 uppercase tracking-tight">{bk.schedule.unwrap_or_else(|| "Manual".into())}</td>
                                                <td class="px-6 py-4 text-sm text-gray-400">{bk.last_run.unwrap_or_else(|| "Never".into())}</td>
                                                <td class="px-6 py-4">
                                                    <BackupStatusBadge status=bk.status />
                                                </td>
                                                <td class="px-6 py-4 text-right">
                                                    <div class="flex items-center justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                                                         <button 
                                                            on:click=move |_| { run_action.dispatch(RunBackupNow { id: id_run.clone() }); }
                                                            disabled=is_viewer
                                                            class=move || format!("p-2 text-gray-400 hover:text-green-400 hover:bg-green-400/10 rounded-lg transition-all {}",
                                                                if is_viewer() { "opacity-0 cursor-not-allowed pointer-events-none" } else { "cursor-pointer" }
                                                            )
                                                            title="Run Now"
                                                        >
                                                            <Play class="w-4 h-4"/>
                                                        </button>
                                                        <button 
                                                            on:click=move |_| { delete_action.dispatch(DeleteBackupConfig { id: id_del.clone() }); }
                                                            disabled=is_viewer
                                                            class=move || format!("p-2 text-gray-400 hover:text-red-400 hover:bg-red-400/10 rounded-lg transition-all {}",
                                                                if is_viewer() { "opacity-0 cursor-not-allowed pointer-events-none" } else { "cursor-pointer" }
                                                            )
                                                            title="Delete"
                                                        >
                                                            <Trash class="w-4 h-4"/>
                                                        </button>
                                                    </div>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view().into_any(),
                                    Err(_) => view! { <tr><td colspan="6" class="px-6 py-8 text-center text-red-400">"Error loading backups."</td></tr> }.into_any(),
                                })}
                            </Suspense>
                        </tbody>
                    </table>
                </div>
            </div>

            {move || show_modal.get().then(|| view! {
                <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-gray-950/80 backdrop-blur-sm animate-in fade-in zoom-in duration-200">
                    <div class="bg-gray-900 border border-gray-800 rounded-2xl shadow-2xl w-full max-w-lg overflow-hidden">
                        <div class="px-8 py-6 border-b border-gray-800 flex items-center justify-between bg-gray-800/30">
                            <h2 class="text-xl font-bold text-gray-100">"Create Backup Task"</h2>
                            <button on:click=move |_| set_show_modal.set(false) class="text-gray-400 hover:text-white transition">"✕"</button>
                        </div>
                        
                        <ActionForm action=create_action attr:class="p-8 space-y-6" on:submit=move |_| set_show_modal.set(false)>
                            <div class="space-y-2">
                                <label class="text-xs font-semibold text-gray-400 uppercase tracking-wider">"Task Name"</label>
                                <input type="text" name="name" required class="w-full px-4 py-2 bg-gray-950 border border-gray-800 rounded-lg focus:ring-1 focus:ring-blue-500 focus:border-blue-500 transition-all text-sm" placeholder="Daily Main DB Backup"/>
                            </div>

                            <div class="grid grid-cols-2 gap-6">
                                <div class="space-y-2">
                                    <label class="text-xs font-semibold text-gray-400 uppercase tracking-wider">"Source Type"</label>
                                    <select name="source_type" class="w-full px-4 py-2 bg-gray-950 border border-gray-800 rounded-lg focus:ring-1 focus:ring-blue-500 focus:border-blue-500 transition-all text-sm">
                                        <option value="Database">"Database"</option>
                                        <option value="Container">"Docker Container"</option>
                                        <option value="VM">"Virtual Machine"</option>
                                        <option value="External">"External Server"</option>
                                    </select>
                                </div>
                                <div class="space-y-2">
                                    <label class="text-xs font-semibold text-gray-400 uppercase tracking-wider">"Backup Schedule"</label>
                                    <select name="schedule" class="w-full px-4 py-2 bg-gray-950 border border-gray-800 rounded-lg focus:ring-1 focus:ring-blue-500 focus:border-blue-500 transition-all text-sm">
                                        <option value="Manual">"Manual Only"</option>
                                        <option value="Daily">"Daily at 00:00"</option>
                                        <option value="Weekly">"Weekly (Sunday)"</option>
                                        <option value="Monthly">"Monthly (1st)"</option>
                                    </select>
                                </div>
                            </div>

                            <div class="space-y-2">
                                <label class="text-xs font-semibold text-gray-400 uppercase tracking-wider">"Select Resource"</label>
                                <select name="source_id" class="w-full px-4 py-2 bg-gray-950 border border-gray-800 rounded-lg focus:ring-1 focus:ring-blue-500 focus:border-blue-500 transition-all text-sm">
                                    <optgroup label="Databases">
                                        {move || databases_res.get().map(|res| match res {
                                            Ok(items) => items.into_iter().map(|i| view! { <option value=i.id.clone()>{i.name}</option> }).collect_view().into_any(),
                                            _ => view! { }.into_any()
                                        })}
                                    </optgroup>
                                    <optgroup label="Containers">
                                        {move || containers_res.get().map(|res| match res {
                                            Ok(items) => items.into_iter().map(|i| view! { <option value=i.id.clone()>{i.name}</option> }).collect_view().into_any(),
                                            _ => view! { }.into_any()
                                        })}
                                    </optgroup>
                                    <optgroup label="VMs">
                                        {move || vms_res.get().map(|res| match res {
                                            Ok(items) => items.into_iter().map(|i| view! { <option value=i.id.clone()>{i.name}</option> }).collect_view().into_any(),
                                            _ => view! { }.into_any()
                                        })}
                                    </optgroup>
                                </select>
                            </div>

                            <div class="space-y-2">
                                <label class="text-xs font-semibold text-gray-400 uppercase tracking-wider">"Destination Path"</label>
                                <input type="text" name="destination" required class="w-full px-4 py-2 bg-gray-950 border border-gray-800 rounded-lg focus:ring-1 focus:ring-blue-500 focus:border-blue-500 transition-all text-sm" value="/var/lib/cloudlab/backups/"/>
                                <p class="text-[10px] text-gray-500">"Backups are stored within Cloudlab's managed storage pool."</p>
                            </div>

                            <div class="pt-4 flex items-center justify-end gap-3">
                                <button type="button" on:click=move |_| set_show_modal.set(false) class="px-4 py-2 text-sm font-semibold text-gray-400 hover:text-white transition">"Cancel"</button>
                                <button type="submit" class="px-6 py-2 bg-purple-600 hover:bg-purple-500 text-white rounded-lg font-semibold transition-all shadow-lg shadow-purple-900/40 cursor-pointer">"Create Task"</button>
                            </div>
                        </ActionForm>
                    </div>
                </div>
            })}
        </div>
    }
}

#[component]
fn BackupStatusBadge(status: String) -> impl IntoView {
    let classes = match status.as_str() {
        "idle" => "bg-gray-500/10 text-gray-400 border-gray-500/20",
        "running" => "bg-blue-500/10 text-blue-400 border-blue-500/20 animate-pulse",
        "success" => "bg-green-500/10 text-green-400 border-green-500/20",
        "failed" => "bg-red-500/10 text-red-400 border-red-500/20",
        _ => "bg-gray-500/10 text-gray-400 border-gray-500/20",
    };
    view! {
        <span class=format!("text-[10px] font-bold px-2 py-0.5 rounded-full border uppercase tracking-wider {}", classes)>
            {status}
        </span>
    }
}
